(ns bits.datahike
  (:require
   [bits.schema :as schema]
   [bits.spec]
   [clojure.spec.alpha :as s]
   [com.stuartsierra.component :as component]
   [datahike-jdbc.core]
   [datahike.api :as d]
   [hasch.core :as hasch]
   [io.pedestal.log :as log]
   [lambdaisland.uri :as uri]
   [steffan-westcott.clj-otel.api.trace.span :as span]))

;;; ----------------------------------------------------------------------------
;;; Store config

(defn- jdbc-url->store
  "Convert a JDBC URL to a Datahike store config."
  ([jdbc-url]
   (jdbc-url->store jdbc-url {}))
  ([jdbc-url options]
   {:pre [(s/valid? ::jdbc-url jdbc-url)]}
   (let [table  (:table options "datahike")
         url    (uri/uri (subs jdbc-url 5))
         params (uri/query-string->map (:query url))
         port   (:port url)
         dbtype (:scheme url)
         host   (:host url)
         dbname (subs (:path url) 1)
         id     (hasch/uuid {:dbtype dbtype :host host :dbname dbname :table table})]
     {:backend  :jdbc
      :id       id
      :dbtype   dbtype
      :host     host
      :port     (cond (int? port)    port
                      (string? port) (parse-long port)
                      :else          nil)
      :dbname   dbname
      :user     (:user params)
      :password (:password params)
      :table    table})))

(defn memory-store
  "Create an in-memory store config for development/testing."
  []
  {:backend :memory
   :id      (random-uuid)})

;;; ----------------------------------------------------------------------------
;;; Schema

(defn ensure-schema!
  [conn]
  (span/with-span! {:name ::ensure-schema!}
    (d/transact conn {:tx-data schema/schema})
    (log/info :msg "Schema installed")))

;;; ----------------------------------------------------------------------------
;;; Connection

(defn connect
  [store]
  (span/with-span! {:name ::connect}
    (try
      (d/create-database {:store store})
      (catch clojure.lang.ExceptionInfo e
        (when-not (= :db-already-exists (:type (ex-data e)))
          (throw e))))
    (d/connect {:store store})))

(defn disconnect
  [conn]
  (span/with-span! {:name ::disconnect}
    (d/release conn)))

;;; ----------------------------------------------------------------------------
;;; Component

(defrecord Database [database-url conn]
  component/Lifecycle
  (start [this]
    (span/with-span! {:name ::start-database}
      (let [store (jdbc-url->store database-url)
            conn  (connect store)]
        (ensure-schema! conn)
        (assoc this :conn conn))))
  (stop [this]
    (span/with-span! {:name ::stop-database}
      (some-> conn disconnect)
      (assoc this :conn nil))))

(defn make-database
  [config]
  {:pre [(s/valid? ::config config)]}
  (map->Database config))

(defmethod print-method Database
  [_ ^java.io.Writer w]
  (.write w "#<datahike.Database>"))

;;; ----------------------------------------------------------------------------
;;; Query helpers

(defn db
  [datahike]
  @(:conn datahike))

(defn transact!
  [datahike tx-data]
  (span/with-span! {:name ::transact!}
    (d/transact (:conn datahike) {:tx-data tx-data})))

(defn pull
  [datahike selector eid]
  (span/with-span! {:name ::pull}
    (d/pull (db datahike) selector eid)))

;; TODO Remove this generated garbage.
(defn q
  [datahike query & inputs]
  (span/with-span! {:name ::q}
    (apply d/q query (db datahike) inputs)))
