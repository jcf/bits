(ns bits.service
  (:require
   [bits.auth :as auth]
   [bits.coerce :as coerce]
   [bits.html :as html]
   [bits.middleware :as mw]
   [bits.middleware.session :as middleware.session]
   [bits.morph :as morph]
   [bits.response]
   [bits.service.creator :as creator]
   [bits.service.platform :as platform]
   [bits.ui :as ui]
   [clojure.core.async :as a]
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
   [steffan-westcott.clj-otel.api.trace.span :as span]))

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
          action      (get-in data [:value :action])
          message     (if action
                        (str "Unknown action: " action)
                        "Invalid request parameters")
          remote-addr (:remote-addr request)]
      (log/warn :msg message :action action :remote-addr remote-addr :errors data)
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
                  (ui/page-title {} "Realm not found")
                  (ui/text-muted {:class ["mt-4"]}
                                 "Want your own Bits? We want to hear from you!")))

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
;;; Actions

(def actions
  (merge
   platform/actions
   {:auth/login    {:handler auth/authenticate
                    :params  [[:email :email]
                              [:password :password]]}
    :auth/sign-out auth/sign-out}))

;;; ----------------------------------------------------------------------------
;;; Routes

(defn- morphable
  ([layout-fn view-fn]
   (morphable layout-fn view-fn {}))
  ([layout-fn view-fn options]
   (let [status #(get-in % [:session/realm :realm/status] 200)]
     {:get  (fn [request]
              {:status  (status request)
               :headers {"content-type" "text/html; charset=utf-8"}
               :body    (html/html (layout-fn request (view-fn request)))})
      :post (morph/render-handler view-fn options)})))

(defn- home-view
  [request]
  (let [view-fn (get-in request [:session/realm :realm/view])]
    (assert (fn? view-fn) "No :realm/view in session realm?!")
    (view-fn request)))

(defn- home-layout
  [request & content]
  (let [layout-fn (get-in request [:session/realm :realm/layout])]
    (assert (fn? layout-fn) "No :realm/layout in session realm?!")
    (apply layout-fn request content)))

(defn- login-view-wrapper
  [request]
  (auth/login-view request {}))

(def routes
  [["/"         (assoc (morphable home-layout home-view)
                       :bits/page (fn [request]
                                    {:page/title (-> request :session/realm :creator/display-name)}))]
   ["/cursors"  (assoc (morphable ui/layout platform/cursors-view {:on-close platform/remove-cursor!})
                       :bits/page {:page/title "Cursors"})]
   ["/counter"  (assoc (morphable ui/layout platform/counter-view)
                       :bits/page {:page/title "Counter"})]
   ["/email"    (assoc (morphable ui/layout platform/email-view)
                       :bits/page {:page/title "Email"})]
   ["/login"    (assoc (morphable ui/layout login-view-wrapper)
                       :bits/page {:page/title "Login"})]
   ["/redirect" (assoc (morphable ui/layout platform/redirect-view)
                       :bits/page {:page/title "Redirect"})]])

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
                refresh-ch
                refresh-mult
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
         [mw/wrap-datomic]
         [middleware.params/wrap-params]
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
         [mw/wrap-secure-headers]]]
    (ring/ring-handler
     (ring/router routes
                  {:data {:coercion   coercion.malli/coercion
                          :middleware [exception-middleware
                                       ring.coercion/coerce-request-middleware
                                       mw/page-middleware]}})
     (ring/routes
      (ring/create-resource-handler {:path "/"})
      (ring/create-default-handler
       {:not-found (fn [request]
                     {:status  404
                      :headers {"content-type" "text/html; charset=utf-8"}
                      :body    (html/html (ui/layout request (ui/not-found-view request)))})}))
     {:middleware middleware})))

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
