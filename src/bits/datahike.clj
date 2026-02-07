(ns bits.datahike
  (:require
   [bits.schema :as schema]
   [bits.spec]
   [clojure.spec.alpha :as s]
   [com.stuartsierra.component :as component]
   [datahike.api :as d]
   [datahike-jdbc.core]
   [hasch.core :as hasch]
   [io.pedestal.log :as log]
   [lambdaisland.uri :as uri]
   [steffan-westcott.clj-otel.api.trace.span :as span]))

;;; ----------------------------------------------------------------------------
;;; Store config

(defn jdbc-url->store
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
  [config]
  {:pre [(s/valid? ::config config)]}
  (span/with-span! {:name ::connect}
    (let [store           (:store config)
          datahike-config {:store store}]
      (log/info :msg "Creating Datahike database..." :config datahike-config)
      (try
        (d/create-database datahike-config)
        (catch clojure.lang.ExceptionInfo e
          (when-not (= :db-already-exists (:type (ex-data e)))
            (throw e))
          (log/debug :msg "Database already exists")))
      (log/info :msg "Connecting to Datahike...")
      (let [conn (d/connect datahike-config)]
        (log/info :msg "Connected to Datahike")
        conn))))

(defn disconnect
  [conn]
  (span/with-span! {:name ::disconnect}
    (d/release conn)))

;;; ----------------------------------------------------------------------------
;;; Component

(defrecord Database [store conn]
  component/Lifecycle
  (start [this]
    (span/with-span! {:name ::start-database}
      (let [conn (connect this)]
        (ensure-schema! conn)
        (assoc this :conn conn))))
  (stop [this]
    (span/with-span! {:name ::stop-database}
      (when conn
        (disconnect conn))
      (assoc this :conn nil))))

(defn make-database
  [config]
  {:pre [(s/valid? ::config config)]}
  (map->Database config))

;;; ----------------------------------------------------------------------------
;;; Query helpers

(defn db
  [database]
  @(:conn database))

(defn transact!
  [database tx-data]
  (span/with-span! {:name ::transact!}
    (d/transact (:conn database) {:tx-data tx-data})))

(defn pull
  [database selector eid]
  (span/with-span! {:name ::pull}
    (d/pull (db database) selector eid)))

(defn q
  [query & inputs]
  (span/with-span! {:name ::q}
    (apply d/q query inputs)))
