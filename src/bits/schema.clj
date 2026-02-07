(ns bits.schema)

;;; ----------------------------------------------------------------------------
;;; User

(def user-schema
  [{:db/ident       :user/id
    :db/valueType   :db.type/uuid
    :db/cardinality :db.cardinality/one
    :db/unique      :db.unique/identity}

   {:db/ident       :user/password-hash
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one}

   {:db/ident       :user/created-at
    :db/valueType   :db.type/instant
    :db/cardinality :db.cardinality/one}])

;;; ----------------------------------------------------------------------------
;;; Email

(def email-schema
  [{:db/ident       :email/address
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one
    :db/unique      :db.unique/identity}

   {:db/ident       :email/user
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/one}

   {:db/ident       :email/verified-at
    :db/valueType   :db.type/instant
    :db/cardinality :db.cardinality/one}

   {:db/ident       :email/preferred?
    :db/valueType   :db.type/boolean
    :db/cardinality :db.cardinality/one}])

;;; ----------------------------------------------------------------------------
;;; Tenant

(def tenant-schema
  [{:db/ident       :tenant/id
    :db/valueType   :db.type/uuid
    :db/cardinality :db.cardinality/one
    :db/unique      :db.unique/identity}

   {:db/ident       :tenant/purpose
    :db/valueType   :db.type/keyword
    :db/cardinality :db.cardinality/one
    :db/unique      :db.unique/identity}

   {:db/ident       :tenant/created-at
    :db/valueType   :db.type/instant
    :db/cardinality :db.cardinality/one}])

;;; ----------------------------------------------------------------------------
;;; Domain

(def domain-schema
  [{:db/ident       :domain/name
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one
    :db/unique      :db.unique/identity}

   {:db/ident       :domain/tenant
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/one}

   {:db/ident       :domain/added-by
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/one}])

;;; ----------------------------------------------------------------------------
;;; Membership

(def membership-schema
  [{:db/ident       :membership/id
    :db/valueType   :db.type/uuid
    :db/cardinality :db.cardinality/one
    :db/unique      :db.unique/identity}

   {:db/ident       :membership/user
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/one}

   {:db/ident       :membership/tenant
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/one}

   {:db/ident       :membership/role
    :db/valueType   :db.type/keyword
    :db/cardinality :db.cardinality/one}])

;;; ----------------------------------------------------------------------------
;;; Full schema

(def schema
  (vec (concat user-schema
               email-schema
               tenant-schema
               domain-schema
               membership-schema)))
