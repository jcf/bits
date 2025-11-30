(ns bits.service
  (:require
   [bits.cors :as cors]
   [bits.csp :as csp]
   [bits.csrf :as csrf]
   [bits.interceptor :as i]
   [bits.router :as router]
   [bits.service.hello :as service.hello]
   [bits.service.session :as service.session]
   [bits.spec]
   [clojure.spec.alpha :as s]
   [clojure.string :as str]
   [com.stuartsierra.component :as component]
   [io.pedestal.connector :as connector]
   [io.pedestal.http.body-params :as http.body-params]
   [io.pedestal.http.http-kit :as http.http-kit]
   [io.pedestal.http.ring-middlewares :as http.middleware]
   [io.pedestal.http.route :as http.route]
   [io.pedestal.http.secure-headers :as http.secure-headers]
   [io.pedestal.interceptor :refer [interceptor]]
   [io.pedestal.log :as log]
   [io.pedestal.service.interceptors :as http.interceptors]
   [ring.middleware.session.cookie :as middleware.session.cookie]
   [steffan-westcott.clj-otel.api.trace.http :as trace.http]
   [steffan-westcott.clj-otel.api.trace.span :as span]))

;;; ----------------------------------------------------------------------------
;;; Routes

(def routes
  (router/combine
   service.hello/routes
   service.session/routes))

;;; ----------------------------------------------------------------------------
;;; URLs

(def ^{:arglists '([route-name & options])}
  url-for
  (http.route/url-for-routes (http.route/expand-routes routes)))

;;; ----------------------------------------------------------------------------
;;; Tracing

(defn- trace-service
  [service-map]
  (update
   service-map :interceptors
   (fn [interceptors]
     (let [is     (trace.http/server-span-interceptors {:create-span? false})
           before (mapv interceptor is)
           after  (interceptor (trace.http/route-interceptor))]
       (conj (into before interceptors) after)))))

;;; ------------------------------------------------------------------------------------------------------------------
;;; CSRF

(defn forged-request-handler
  [{:keys [request] :as context}]
  (log/warn :msg            "Invalid anti-forgery token?!"
            :request-method (:request-method request)
            :uri            (:uri request)
            :form-params    (:form-params request))
  (assoc context :response
         {:status  403
          :headers {"Content-Type" "text/plain"}
          :body    "Unauthorized request.\n"}))

(defn- request-verifier
  [request]
  (get-in request [:form-params :verifier]))

;;; ----------------------------------------------------------------------------
;;; Component

(defn make-connector
  [service]
  (let [{:keys [allow-credentials?
                allowed-headers
                allowed-origins
                canonical-host
                cookie-name
                cookie-secret
                env
                reload-assets?]
         :or   {env :prod}} service
        cors                {:allow-credentials? allow-credentials?
                             :allowed-headers    allowed-headers
                             :allowed-origins    allowed-origins
                             :routes             routes}
        ;; Guard against Ring generating a random cookie secret and breaking all
        ;; of our sessions.
        _                   (s/assert bytes? cookie-secret)
        cookie-store        (middleware.session.cookie/cookie-store {:key cookie-secret})
        session             {:cookie-attrs {:http-only true
                                            :secure    true}
                             :cookie-name  cookie-name
                             :store        cookie-store}
        interceptors
        (filterv
         some?
         [http.interceptors/log-request
          http.route/query-params
          i/error-interceptor
          i/not-found-interceptor
          (i/make-state-interceptor service)
          (cors/make-cors-interceptor cors)
          (http.body-params/body-params)
          (http.middleware/content-type {:mime-types {}})
          (http.middleware/session session)
          (csrf/anti-forgery {:error-handler forged-request-handler
                              :read-token    request-verifier})
          i/asset-interceptor
          (when reload-assets? i/reload-assets-interceptor)
          (i/make-referrer-policy-interceptor)
          (http.secure-headers/secure-headers
           {:content-security-policy-settings (csp/csp-map->str (csp/policy))})
          (http.middleware/resource "public")])]

    (-> {:host            (:http-host service)
         :initial-context {}
         :interceptors    interceptors
         :join?           false
         :port            (:http-port service)
         :router          :sawtooth}
        trace-service
        (connector/with-routes routes))))

(defrecord Service [allow-credentials?
                    allowed-headers
                    allowed-origins
                    buster
                    commit-id
                    connector
                    cookie-name
                    cookie-secret
                    http-host
                    http-port
                    join?
                    reload-assets?
                    server-header]
  component/Lifecycle
  (start [this]
    (span/with-span! {:name ::start-service}
      (let [connector (http.http-kit/create-connector
                       (make-connector this) {:server-header server-header})]
        (connector/start! connector)
        (assoc this :connector connector))))
  (stop [this]
    (span/with-span! {:name ::stop-service}
      (some-> this :connector connector/stop!)
      (assoc this :connector nil))))

(defmethod print-method Service
  [service ^java.io.Writer w]
  (.write w (format "#<Service port=%s>" (:http-port service))))

(s/fdef make-service
  :args (s/cat :config ::config)
  :ret ::config)

(defn make-service
  [config]
  (map->Service config))
