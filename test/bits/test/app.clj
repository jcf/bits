(ns bits.test.app
  (:require
   [bits.app :as app]
   [bits.cryptex :as cryptex]
   [bits.crypto :as crypto]
   [bits.datomic :as datomic]
   [bits.postgres :as postgres]
   [bits.test.postgres :as test.postgres]
   [clojure.spec.alpha :as s]
   [clojure.string :as str]
   [com.stuartsierra.component :as component]
   [hato.client :as http]
   [java-time.api :as time]
   [ring.util.response :as response])
  (:import
   (java.net CookieManager CookiePolicy)))

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
                                    (assoc-in [:datomic :uri] (str "datomic:mem://bits-test-" (random-uuid)))
                                    (assoc-in [:postgres :database-url] ephemeral-url)
                                    (assoc-in [:service :cookie-name] "bits")
                                    (assoc-in [:service :cookie-secure] false)
                                    (assoc-in [:service :csrf-cookie-name] "bits-csrf")
                                    (assoc-in [:service :http-port] 0))
        ephemeron               (test.postgres/make-ephemeron {:database-url  ephemeral-url
                                                               :template-name template-name})
        deps                    (-> app/dependencies
                                    (update :datomic (fnil conj []) :ephemeron)
                                    (update :migrator (fnil conj []) :ephemeron)
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
  (let [port (service-port service)]
    (str "http://localhost"
         (when port (str ":" port))
         (str/replace-first path #"^/?" "/"))))

;;; ----------------------------------------------------------------------------
;;; Hosts

(defn host
  [request host]
  (response/header request "host" host))

;;; ----------------------------------------------------------------------------
;;; Request

(defn cookie-manager
  []
  (CookieManager. nil CookiePolicy/ACCEPT_ALL))

(defn http-client
  ([] (http-client {}))
  ([options]
   (http/build-http-client (merge {:connect-timeout 100} options))))

(defn- cleanup-hato-response
  [response]
  (-> response
      (select-keys #{:body :headers :status})
      (update :headers #(into (sorted-map) (dissoc % ":status")))))

(defn request
  [service request-options]
  (-> (merge {:http-client       (http-client)
              :throw-exceptions? false} request-options)
      (update :url #(service-url service %))
      http/request
      cleanup-hato-response))

;;; ----------------------------------------------------------------------------
;;; Users

(defn- user-txes
  [email password-hash]
  [{:user/id            (random-uuid)
    :user/email         email
    :user/password-hash password-hash
    :user/created-at    (time/java-date)}])

(defn- hash-password
  [keymaster password]
  (crypto/derive keymaster (cryptex/cryptex password)))

(defn create-user!
  [service email password]
  (let [{:keys [datomic
                keymaster]} service
        txes                (user-txes email (hash-password keymaster password))]
    (datomic/transact! datomic txes)))
