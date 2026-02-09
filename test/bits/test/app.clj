(ns bits.test.app
  (:require
   [bits.app :as app]
   [bits.cryptex :as cryptex]
   [bits.crypto :as crypto]
   [bits.datahike :as datahike]
   [bits.postgres :as postgres]
   [bits.test.postgres :as test.postgres]
   [clojure.spec.alpha :as s]
   [clojure.string :as str]
   [com.stuartsierra.component :as component]
   [datahike.core]
   [hato.client :as http]))

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
  (let [config                  (app/read-config)
        database-url            (postgres/replace-dbname
                                 (get-in config [:postgres :database-url]) "bits_test")
        {:keys [ephemeral-url
                template-name]} (test.postgres/ephemerize database-url)
        config                  (-> config
                                    (assoc-in [:service :http-port] 0)
                                    (assoc-in [:postgres :database-url] ephemeral-url)
                                    (assoc-in [:datahike :database-url] ephemeral-url))
        ephemeron               (test.postgres/make-ephemeron {:database-url  ephemeral-url
                                                               :template-name template-name})
        deps                    (-> app/dependencies
                                    (update :datahike (fnil conj []) :ephemeron)
                                    (update :postgres (fnil conj []) :ephemeron))]
    (-> config
        app/components
        (assoc :ephemeron ephemeron)
        (component/system-using deps)
        (component/subsystem #{:service}))))

(defn replace-allowed-origins
  [system origins]
  (assoc-in system [:service :allowed-origins] origins))

(defn replace-random-bytes
  [system source]
  (assoc system :randomizer (reify crypto/Randomize
                              (random-bytes [_ size] (source size)))))

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

;;; ----------------------------------------------------------------------------
;;; Request

(def ^:private http-client
  (http/build-http-client {:connect-timeout 100}))

(defn- cleanup-hato-response
  [response]
  (-> response
      (select-keys #{:body :headers :status})
      (update :headers #(into (sorted-map) (dissoc % ":status")))))

(defn request
  [service request-options]
  (-> (merge {:throw-exceptions? false} request-options)
      (assoc :http-client http-client)
      (update :url #(service-url service %))
      http/request
      cleanup-hato-response))

;;; ----------------------------------------------------------------------------
;;; Users

(defn- user-txes
  [email password-hash]
  (let [tempid (datahike.core/tempid :db.part/ignored)]
    [{:db/id              tempid
      :user/id            (random-uuid)
      :user/password-hash password-hash
      :user/created-at    (java.util.Date.)}
     {:email/address    email
      :email/user       tempid
      :email/preferred? true}]))

(defn- hash-password
  [keymaster password]
  (crypto/derive keymaster (cryptex/cryptex password)))

(defn create-user!
  [service email password]
  (let [{:keys [datahike
                keymaster]} service
        txes                (user-txes email (hash-password keymaster password))]
    (datahike/transact! datahike txes)))
