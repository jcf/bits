(ns bits.plane-test
  (:require
   [bits.cryptex :as cryptex]
   [bits.plane :as sut]
   [clojure.test :refer [deftest is]]
   [com.stuartsierra.component :as component]
   [datomic.api :as d]))

;;; ----------------------------------------------------------------------------
;;; Fixtures
;;;
;;; Schema is installed by the test through the plane's API — deliberately not
;;; by the plane itself.

(def ^:private control-schema
  [{:db/ident       :tenant/id
    :db/valueType   :db.type/uuid
    :db/cardinality :db.cardinality/one
    :db/unique      :db.unique/identity}
   {:db/ident       :tenant/domains
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/many}
   {:db/ident       :domain/name
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one
    :db/unique      :db.unique/identity}])

(def ^:private tenant-schema
  [{:db/ident       :creator/handle
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one}])

(def ^:private tenant-by-domain-query
  '[:find ?tenant-id .
    :in $ ?domain
    :where
    [?d :domain/name ?domain]
    [?t :tenant/domains ?d]
    [?t :tenant/id ?tenant-id]])

(def ^:private handle-query
  '[:find ?handle .
    :where [_ :creator/handle ?handle]])

(defn- started-plane
  []
  (let [base (str "datomic:mem://plane-test-" (random-uuid))]
    (component/start
     (sut/make-plane {:control-uri         (cryptex/cryptex base)
                      :max-conns           8
                      :release-timeout-ms  5000
                      :tenant-uri-template (cryptex/cryptex (str base "-tenant-%s"))}))))

(defn- provisioned-realm!
  [plane domain-name]
  (let [tenant-id (random-uuid)]
    (sut/provision! plane {:domain/name domain-name :tenant/id tenant-id})
    (sut/transact! plane tenant-id tenant-schema)
    tenant-id))

;;; ----------------------------------------------------------------------------
;;; tenant-uri

(deftest tenant-uri
  (let [plane (sut/make-plane {:control-uri         (cryptex/cryptex "datomic:mem://control")
                               :max-conns           8
                               :release-timeout-ms  1000
                               :tenant-uri-template (cryptex/cryptex "datomic:sql://tenant-%s?jdbc:x")})]
    (is (= "datomic:sql://tenant-abc?jdbc:x"
           (cryptex/reveal (sut/tenant-uri plane "abc"))))))

;;; ----------------------------------------------------------------------------
;;; provision!

(deftest provision!
  (let [plane (started-plane)]
    (try
      (sut/transact-control! plane control-schema)
      (let [a (provisioned-realm! plane "a.bits.page")
            b (provisioned-realm! plane "b.bits.page")]
        (is (= a (d/q tenant-by-domain-query (sut/control-db plane) "a.bits.page")))
        (is (= b (d/q tenant-by-domain-query (sut/control-db plane) "b.bits.page")))
        (is (nil? (d/q tenant-by-domain-query (sut/control-db plane) "unknown.example"))))
      (finally
        (component/stop plane)))))

;;; ----------------------------------------------------------------------------
;;; with-db / transact!

(deftest with-db
  (let [plane (started-plane)]
    (try
      (sut/transact-control! plane control-schema)
      (let [a (provisioned-realm! plane "a.bits.page")
            b (provisioned-realm! plane "b.bits.page")]
        (sut/transact! plane a [{:creator/handle "alice"}])
        (is (= "alice" (sut/with-db [db plane a] (d/q handle-query db))))
        (is (nil? (sut/borrow-db plane b (fn find-handle [db] (d/q handle-query db))))
            "Tenant databases are isolated."))
      (finally
        (component/stop plane)))))
