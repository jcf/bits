(ns bits.test.app
  (:require
   [bits.app :as app]
   [clojure.spec.alpha :as s]
   [clojure.string :as str]
   [com.stuartsierra.component :as component]
   [io.pedestal.log :as log]
   [steffan-westcott.clj-otel.api.trace.span :as span]))

(defn- system-ex-info
  [cause]
  (let [origin (last (take-while some? (iterate ex-cause cause)))]
    (ex-info (format "Error starting system: %s" (ex-message origin))
             (ex-data origin)
             cause)))

(defn must-start-system
  [system-map]
  (try
    (component/start-system system-map)
    (catch Exception cause
      (some-> cause ex-data :system component/stop-system)
      (throw (system-ex-info cause)))))

(defmacro with-system
  {:arglists     ['([system-binding system-map] body*)]
   :style/indent 1}
  [& more]
  (let [[[system-binding system-map] & body] more]
    `(let [running#        (must-start-system ~system-map)
           ~system-binding running#]
       (try
         ~@body
         (finally
           (component/stop-system running#))))))

(defn system
  []
  (-> (app/read-config)
      (assoc-in [:service :http-port] 0)
      app/components
      (component/system-using app/dependencies)
      (component/subsystem #{:service})))

(defn replace-allowed-origins
  [system origins]
  (assoc-in system [:service :allowed-origins] origins))

;;; ----------------------------------------------------------------------------
;;; URLs

(defn service-port
  [service]
  (some-> service :stop-fn meta :local-port))

(s/fdef service-url
  :args (s/cat :service ::service :path string?)
  :ret  string?)

(defn service-url
  [service path]
  (let [port (service-port service)
        ;; http-kit always binds to the configured host, defaulting to "0.0.0.0"
        ;; For test URLs, always use localhost
        host "localhost"]
    (str "http://" host
         (when port (str ":" port))
         (str/replace-first path #"^/?" "/"))))
