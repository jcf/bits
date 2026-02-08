(ns bits.service
  (:require
   [bits.coerce :as coerce]
   [bits.middleware :as mw]
   [bits.morph :as morph]
   [clojure.core.async :as a]
   [com.stuartsierra.component :as component]
   [org.httpkit.server :as server]
   [reitit.coercion.malli :as coercion.malli]
   [reitit.ring :as ring]
   [reitit.ring.coercion :as ring.coercion]
   [ring.middleware.cookies :as middleware.cookies]
   [ring.middleware.params :as middleware.params]
   [ring.middleware.session :as middleware.session]
   [steffan-westcott.clj-otel.api.trace.span :as span]))

;;; ----------------------------------------------------------------------------
;;; App

(defn make-app
  "Builds Ring handler. Normalizes actions and builds schema at startup.
   Routes and actions come from service fields - pass a test service for testing."
  [service]
  (let [{:keys [actions
                channels
                cookie-name
                csrf-cookie-name
                csrf-secret
                refresh-ch
                refresh-mult
                routes
                session-store]} service

        actions       (morph/normalize-actions actions)
        action-schema (morph/actions->schema actions)
        routes        (conj routes
                            ["/action"
                             {:post {:coercion   coerce/coercion
                                     :parameters {:form action-schema}
                                     :handler    (morph/action-handler actions)}}])

        middleware
        [[morph/wrap-refresh refresh-ch refresh-mult]
         [morph/wrap-channels channels]
         [mw/wrap-state service]
         [middleware.params/wrap-params]
         [middleware.cookies/wrap-cookies]
         [middleware.session/wrap-session {:cookie-attrs {:http-only true
                                                          :same-site :lax
                                                          :secure    true}
                                           :cookie-name  cookie-name
                                           :store        session-store}]
         [morph/wrap-csrf {:cookie-name csrf-cookie-name
                           :secret      csrf-secret}]
         [mw/wrap-secure-headers]]]
    (ring/ring-handler
     (ring/router routes
                  {:data {:coercion   coercion.malli/coercion
                          :middleware [ring.coercion/coerce-request-middleware]}})
     (ring/routes
      (ring/create-resource-handler {:path "/"}))
     {:middleware middleware})))

;;; ----------------------------------------------------------------------------
;;; Service

(defrecord Service [actions
                    channels
                    cookie-name
                    csrf-cookie-name
                    csrf-secret
                    datahike
                    http-host
                    http-port
                    keymaster
                    max-refresh-ms
                    pool
                    refresh-ch
                    refresh-mult
                    routes
                    server-name
                    session-store
                    stop-fn]
  component/Lifecycle
  (start [this]
    (span/with-span! {:name ::start-service}
      (let [channels     (atom {})
            refresh-ch   (a/chan (a/dropping-buffer 1))
            throttled    (if max-refresh-ms
                           (morph/throttle refresh-ch max-refresh-ms)
                           refresh-ch)
            refresh-mult (a/mult throttled)
            this         (assoc this
                                :channels     channels
                                :refresh-ch   refresh-ch
                                :refresh-mult refresh-mult)]
        (assoc this :stop-fn (server/run-server (make-app this)
                                                {:host                       http-host
                                                 :legacy-unsafe-remote-addr? false
                                                 :port                       http-port
                                                 :server-header              server-name})))))
  (stop [this]
    (span/with-span! {:name ::stop-service}
      (when-let [channels (:channels this)]
        (doseq [[_ {:keys [close!]}] @channels]
          (close!))
        (reset! channels {}))
      (when-let [stop (:stop-fn this)]
        (stop :timeout 200))
      (when-let [ch (:refresh-ch this)]
        (a/close! ch))
      (assoc this :channels nil :refresh-ch nil :refresh-mult nil :stop-fn nil))))

(defn make-service
  [config]
  (map->Service config))

;;; ----------------------------------------------------------------------------
;;; Utilities

(defn refresh!
  [service]
  (a/put! (:refresh-ch service) :action))

(defn stats
  [service]
  (let [channels @(:channels service)]
    {:channels (count channels)
     :sessions (count (into #{} (map :sid) channels))}))
