(ns bits.plane
  "Owns the control-plane database connection and the lifecycle of per-tenant
  databases. Database values and transaction data flow in the open so callers
  build queries and transactions as data and execute them intentionally; the
  plane mediates connection lifecycle only. Schema installation is
  deliberately not the plane's concern while the schema versioning story is
  designed."
  (:require
   [bits.cryptex :as cryptex]
   [bits.datomic :as datomic]
   [bits.lender :as lender]
   [bits.spec]
   [clojure.spec.alpha :as s]
   [com.stuartsierra.component :as component]
   [datomic.api :as d]
   [steffan-westcott.clj-otel.api.trace.span :as span]))

;;; ----------------------------------------------------------------------------
;;; URIs

(defn expand-tenant-uri
  [tenant-uri-template tenant-id]
  {:post [(datomic/datomic-uri? (cryptex/reveal %))]}
  (cryptex/cryptex (format (cryptex/reveal tenant-uri-template) tenant-id)))

(defn- expands-distinctly?
  [tenant-uri-template]
  (not= (expand-tenant-uri tenant-uri-template "a")
        (expand-tenant-uri tenant-uri-template "b")))

(defn tenant-uri
  [plane tenant-id]
  {:pre [(s/valid? ::plane plane)]}
  (expand-tenant-uri (:tenant-uri-template plane) tenant-id))

;;; ----------------------------------------------------------------------------
;;; Component

(defrecord Plane [conn conns control-uri max-conns release-timeout-ms tenant-uri-template]
  component/Lifecycle
  (start [this]
    (span/with-span! {:name ::start}
      (assoc this
             :conn  (datomic/connect (cryptex/reveal control-uri))
             :conns (lender/make-cache {:max-size max-conns}))))
  (stop [this]
    (span/with-span! {:name ::stop}
      (some-> conns lender/drain!)
      (some->> conn (datomic/release-conn release-timeout-ms control-uri))
      (assoc this :conn nil :conns nil))))

(defmethod print-method Plane
  [_ ^java.io.Writer w]
  (.write w "#<Plane>"))

(defn plane?
  [x]
  (instance? Plane x))

(s/def ::plane plane?)

(defn make-plane
  [config]
  {:pre [(s/valid? ::config config)
         (expands-distinctly? (:tenant-uri-template config))]}
  (map->Plane config))

;;; ----------------------------------------------------------------------------
;;; Control plane

(defn control-db
  [plane]
  {:pre [(s/valid? ::plane plane) (some? (:conn plane))]}
  (d/db (:conn plane)))

(defn transact-control!
  [plane tx-data]
  {:pre [(s/valid? ::plane plane) (some? (:conn plane))]}
  (span/with-span! {:name ::transact-control!}
    @(d/transact (:conn plane) tx-data)))

;;; ----------------------------------------------------------------------------
;;; Tenant databases

(defn- lend-conn
  [plane uri func]
  (let [{:keys [conns release-timeout-ms]} plane]
    (lender/lend conns datomic/open-conn
                 (partial datomic/release-conn release-timeout-ms)
                 uri func)))

(defn borrow-db
  [plane tenant-id func]
  {:pre [(s/valid? ::plane plane)]}
  (span/with-span! {:name ::borrow-db :attributes {"tenant_id" (str tenant-id)}}
    (lend-conn plane (tenant-uri plane tenant-id) (comp func d/db))))

(defmacro with-db
  "binding => [db-sym plane tenant-id]"
  [binding & body]
  (let [[sym plane tenant-id] binding]
    `(borrow-db ~plane ~tenant-id (^:once fn* [~sym] ~@body))))

(defn transact!
  [plane tenant-id tx-data]
  {:pre [(s/valid? ::plane plane)]}
  (span/with-span! {:name ::transact! :attributes {"tenant_id" (str tenant-id)}}
    (lend-conn plane (tenant-uri plane tenant-id)
               (fn transact-tenant [conn] @(d/transact conn tx-data)))))

(defn provision!
  "Registers the realm in the control plane and creates its database. Schema
  installation is deliberately left to the caller while the schema versioning
  story is designed."
  [plane realm]
  {:pre [(plane? plane) (s/valid? ::realm realm)]}
  (let [{domain-name :domain/name tenant-id :tenant/id} realm]
    (span/with-span! {:name       ::provision!
                      :attributes {"domain_name" domain-name
                                   "tenant_id"   (str tenant-id)}}
      (transact-control! plane [{:tenant/id      tenant-id
                                 :tenant/domains [{:domain/name domain-name}]}])
      (d/create-database (cryptex/reveal (tenant-uri plane tenant-id)))
      tenant-id)))
