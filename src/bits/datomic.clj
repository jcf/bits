(ns bits.datomic
  (:require
   [bits.schema :as schema]
   [bits.spec]
   [clojure.spec.alpha :as s]
   [com.stuartsierra.component :as component]
   [datomic.api :as d]
   [io.pedestal.log :as log]
   [steffan-westcott.clj-otel.api.trace.span :as span]))

;;; ----------------------------------------------------------------------------
;;; Schema

(defn ensure-schema!
  [conn]
  (span/with-span! {:name ::ensure-schema!}
    @(d/transact conn schema/schema)
    (log/info :msg "Schema installed.")))

;;; ----------------------------------------------------------------------------
;;; Connection

(defn connect
  [uri]
  (span/with-span! {:name ::connect}
    (d/create-database uri)
    (d/connect uri)))

(defn disconnect
  [conn]
  (span/with-span! {:name ::disconnect}
    (d/release conn)))

(defn conn
  [datomic]
  {:post [(some? %)]}
  (:conn datomic))

;;; ----------------------------------------------------------------------------
;;; Database

(defn db
  [datomic]
  (-> datomic conn d/db))

;;; ----------------------------------------------------------------------------
;;; Component

(defrecord Datomic [conn uri]
  component/Lifecycle
  (start [this]
    (span/with-span! {:name ::start-database}
      (let [conn (connect uri)]
        (ensure-schema! conn)
        (assoc this :conn conn))))
  (stop [this]
    (span/with-span! {:name ::stop-database}
      (some-> conn disconnect)
      (assoc this :conn nil))))

(defmethod print-method Datomic
  [_ ^java.io.Writer w]
  (.write w "#<Datomic>"))

(defn make-datomic
  [config]
  {:pre [(s/valid? ::config config)]}
  (map->Datomic config))
