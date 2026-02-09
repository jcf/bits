(ns bits.test.postgres
  (:require
   [bits.postgres :as postgres]
   [clojure.string :as str]
   [com.stuartsierra.component :as component]
   [io.pedestal.log :as log]
   [next.jdbc :as jdbc]))

(defn- slash
  [s]
  (str/replace-first s #"^/?" "/"))

(defn- ephemeral-name
  [s]
  (str s "_" (Long/toString (System/currentTimeMillis) 36)))

(defn ephemerize
  [db-url]
  (let [url     (postgres/parse-url db-url)
        db-name (:path url)]
    {:ephemeral-url (str "jdbc:" (update url :path (comp slash ephemeral-name)))
     :template-name (subs db-name 1)}))

(comment
  (ephemeral-name "database")
  (ephemerize "jdbc:postgresql://127.0.0.1:5432/bits_test?user=bits&password=please"))

(defn- create-from-template!
  [conn template-name db-name]
  (jdbc/execute-one! conn [(format "create database %s template %s"
                                   (postgres/strop \" db-name \")
                                   (postgres/strop \" template-name \"))]))

(defn- drop-database!
  [conn db-name]
  (jdbc/execute-one! conn [(format "drop database %s with (force)"
                                   (postgres/strop \" db-name \"))]))

(defrecord Ephemeron [conn database-url template-name]
  component/Lifecycle
  (start [this]
    (let [url  (postgres/replace-dbname database-url "postgres")
          conn (jdbc/get-connection url {:auto-commit true})]
      (try
        (log/debug :msg           "Creating ephemeral database..."
                   :database-url  database-url
                   :template-name template-name)
        (create-from-template! conn template-name (postgres/dbname database-url))
        (catch Exception ex
          (.close conn)
          (throw ex)))
      (assoc this :conn conn)))
  (stop [this]
    (when-let [conn (:conn this)]
      (log/debug :msg           "Dropping ephemeral database..."
                 :database-url  database-url
                 :template-name template-name)
      (drop-database! conn (postgres/dbname database-url))
      (.close conn))
    (assoc this :conn nil)))

(defmethod print-method Ephemeron
  [ephemeron ^java.io.Writer w]
  (.write w (format "#<Ephemeron %s>" (some-> ephemeron :database-url postgres/dbname))))

(defn make-ephemeron
  [config]
  (map->Ephemeron config))

(comment
  (let [database-url "jdbc:postgresql://127.0.0.1:5432/bits_dev?user=bits&password=please"
        ephemeron    (map->Ephemeron {:database-url  database-url
                                      :template-name "bits_test"})]
    (try
      (component/start ephemeron)
      (finally
        (component/stop ephemeron)))))
