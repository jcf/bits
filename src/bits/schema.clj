(ns bits.schema)

;;; ----------------------------------------------------------------------------
;;; User

(def user-schema
  [{:db/ident       :user/id
    :db/valueType   :db.type/uuid
    :db/cardinality :db.cardinality/one
    :db/unique      :db.unique/identity}

   {:db/ident       :user/email
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one
    :db/unique      :db.unique/identity}

   {:db/ident       :user/password-hash
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one}

   {:db/ident       :user/created-at
    :db/valueType   :db.type/instant
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
    :db/cardinality :db.cardinality/one}

   {:db/ident       :tenant/domains
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/many}])

;;; ----------------------------------------------------------------------------
;;; Domain

(def domain-schema
  [{:db/ident       :domain/name
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one
    :db/unique      :db.unique/identity}])

;;; ----------------------------------------------------------------------------
;;; Creator

(def creator-schema
  [{:db/ident       :creator/handle
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one
    :db/unique      :db.unique/identity}

   {:db/ident       :creator/display-name
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one}

   {:db/ident       :creator/bio
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one}

   {:db/ident       :creator/avatar-url
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one}

   {:db/ident       :creator/banner-url
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one}

   {:db/ident       :creator/links
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/many
    :db/isComponent true}

   {:db/ident       :creator/posts
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/many}

   {:db/ident       :link/label
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one}

   {:db/ident       :link/url
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one}

   {:db/ident       :link/icon
    :db/valueType   :db.type/keyword
    :db/cardinality :db.cardinality/one}])

;;; ----------------------------------------------------------------------------
;;; Post

(def post-schema
  [{:db/ident       :post/id
    :db/valueType   :db.type/uuid
    :db/cardinality :db.cardinality/one
    :db/unique      :db.unique/identity}

   {:db/ident       :post/text
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one}

   {:db/ident       :post/created-at
    :db/valueType   :db.type/instant
    :db/cardinality :db.cardinality/one}

   {:db/ident       :post/image-url
    :db/valueType   :db.type/string
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
               tenant-schema
               domain-schema
               creator-schema
               post-schema
               membership-schema)))
