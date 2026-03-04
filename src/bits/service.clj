(ns bits.service
  (:require
   [bits.coerce :as coerce]
   [bits.form :as form]
   [bits.html :as html]
   [bits.locale :refer [tru]]
   [bits.middleware :as mw]
   [bits.middleware.session :as middleware.session]
   [bits.module.creator :as creator]
   [bits.module.platform :as platform]
   [bits.module.session :as session]
   [bits.morph :as morph]
   [bits.response]
   [bits.ui :as ui]
   [clojure.core.async :as a]
   [clojure.spec.alpha :as s]
   [com.stuartsierra.component :as component]
   [io.pedestal.log :as log]
   [medley.core :as medley]
   [org.httpkit.server :as server]
   [reitit.coercion :as coercion]
   [reitit.coercion.malli :as coercion.malli]
   [reitit.ring :as ring]
   [reitit.ring.coercion :as ring.coercion]
   [reitit.ring.middleware.exception :as exception]
   [ring.middleware.cookies :as middleware.cookies]
   [ring.middleware.params :as middleware.params]
   [steffan-westcott.clj-otel.api.trace.http :as trace.http]
   [steffan-westcott.clj-otel.api.trace.span :as span])
  (:import
   (java.util.concurrent Executors)))

;;; ----------------------------------------------------------------------------
;;; Exception handling

(defn- default-error-handler
  [exception request]
  (log/error :msg       "Unhandled exception?!"
             :uri       (:uri request)
             :exception exception)
  bits.response/internal-server-error-response)

(defn- coercion-error-handler
  [status]
  (fn [exception request]
    (let [data        (ex-data exception)
          errors      (:errors data)
          action      (get-in data [:transformed :action])
          raw-action  (get-in data [:value :action])
          missing     (->> errors
                           (filter #(= :malli.core/missing-key (:type %)))
                           (map #(last (:path %))))
          message     (cond
                        (seq missing)
                        (str "Missing required fields: " (pr-str missing))

                        (and raw-action (not action))
                        (str "Unknown action: " raw-action)

                        :else
                        "Invalid request parameters")
          remote-addr (:remote-addr request)]
      (log/warn :msg message :action (or action raw-action) :remote-addr remote-addr :errors data)
      {:status status
       :body   message})))

(def exception-middleware
  (exception/create-exception-middleware
   (merge
    exception/default-handlers
    {::exception/default          default-error-handler
     ::coercion/request-coercion  (coercion-error-handler 400)
     ::coercion/response-coercion (coercion-error-handler 500)})))

;;; ----------------------------------------------------------------------------
;;; Realms

(def ^:const ^:private platform-tenant-id
  #uuid "00000000-0000-0000-0000-000000000000")

(def ^:const ^:private unknown-tenant-id
  #uuid "00000000-0000-0000-0000-100000000000")

(defn- realm-not-found-view
  [_request]
  (ui/page-center {}
    (ui/page-title {} (tru "Realm not found"))
    (ui/text-muted {:class ["mt-4"]}
      (tru "Want your own Bits? We want to hear from you!"))))

(def realms
  (medley/index-by
   :realm/type
   #{{:realm/layout ui/layout
      :realm/type   :realm.type/creator
      :realm/view   creator/creator-profile-view}
     {:realm/layout ui/layout
      :realm/type   :realm.type/platform
      :realm/view   platform/explore-view
      :tenant/id    platform-tenant-id}
     {:realm/layout ui/layout
      :realm/status 404
      :realm/type   :realm.type/unknown
      :realm/view   realm-not-found-view
      :tenant/id    unknown-tenant-id}}))

;;; ----------------------------------------------------------------------------
;;; Modules

(def modules
  [creator/module
   platform/module
   session/module])

;;; ----------------------------------------------------------------------------
;;; Broadcast

(defn broadcast!
  [service event]
  (doseq [[_ {:keys [send!]}] @(:channels service)]
    (send! event)))

;;; ----------------------------------------------------------------------------
;;; App

(defn make-app
  "Builds Ring handler. Normalizes actions and builds schema at startup."
  [service]
  (let [{:keys [channels
                cookie-name
                cookie-secure
                csrf-cookie-name
                csrf-secret
                modules
                refresh-ch
                refresh-mult
                session-store]} service

        not-found-handler
        (fn [request]
          {:status  404
           :headers {"content-type" "text/html; charset=utf-8"}
           :body    (html/html (ui/layout request (ui/not-found-view request)))})

        _             (s/assert :bits.module/combined modules)
        actions       (:actions modules)
        action-schema (morph/actions->schema actions)
        routes        (conj (:routes modules)
                            ["/action"
                             {:post {:coercion   coerce/coercion
                                     :parameters {:form action-schema}
                                     :handler    (morph/action-handler actions)}}])

        router
        (ring/router
         routes
         {:data {:coercion   coercion.malli/coercion
                 :middleware [trace.http/wrap-reitit-route
                              exception-middleware
                              ring.coercion/coerce-request-middleware
                              mw/page-middleware]}})

        handler
        (ring/routes (ring/create-default-handler {:not-found not-found-handler}))

        middleware
        [[morph/wrap-refresh refresh-ch refresh-mult]
         [morph/wrap-channels channels]
         [mw/wrap-state service]
         [mw/wrap-datomic]
         [middleware.params/wrap-params]
         [form/wrap-form-params]
         [middleware.cookies/wrap-cookies]
         [mw/wrap-realm realms]
         [middleware.session/wrap-session {:cookie-attrs {:http-only true
                                                          :same-site :lax
                                                          :secure    cookie-secure}
                                           :cookie-name  cookie-name
                                           :store        session-store}]
         [mw/wrap-ensure-session]
         [mw/wrap-csrf {:cookie-name   csrf-cookie-name
                        :cookie-secure cookie-secure
                        :secret        csrf-secret}]
         [mw/wrap-assets]
         [mw/wrap-user]
         [mw/wrap-secure-headers]
         [mw/wrap-locale]]]
    (-> (ring/ring-handler router handler {:middleware middleware})
        (trace.http/wrap-server-span {:create-span? true}))))

;;; ----------------------------------------------------------------------------
;;; Service

(defrecord Service [channels
                    cookie-name
                    cookie-secure
                    csrf-cookie-name
                    csrf-secret
                    datomic
                    http-host
                    http-port
                    keymaster
                    max-refresh-ms
                    modules
                    postgres
                    refresh-ch
                    refresh-mult
                    server-name
                    session-store
                    sse-reconnect-ms
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
        (set-agent-send-executor! (Executors/newVirtualThreadPerTaskExecutor))
        (set-agent-send-off-executor! (Executors/newVirtualThreadPerTaskExecutor))
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

(defmethod print-method Service
  [_ ^java.io.Writer w]
  (.write w "#<Service>"))

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
