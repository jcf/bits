(ns bits.spec
  (:require
   [clojure.spec.alpha :as s]
   [clojure.string :as str]
   [ring.core.spec]))

;;; ----------------------------------------------------------------------------
;;; cljr-clean-ns

(def retain
  "Prevent `cljr-clean-ns` from wiping out the all-important side-effectful
  require of this file by referring to this var."
  ::retain)

;;; ----------------------------------------------------------------------------
;;; Buster

(s/def :bits.assets/resources (s/coll-of string? :kind set?))

(s/def :bits.assets/config
  (s/keys :req-un [:bits.assets/resources]))

;;; ----------------------------------------------------------------------------
;;; Service

(s/def :bits.next/cookie-secret bytes?)
(s/def :bits.next/http-host string?)
(s/def :bits.next/http-port (s/or :zero zero? :pos-int pos-int?))

(s/def :bits.next/config
  (s/keys :req-un [:bits.next/cookie-secret
                   :bits.next/http-host
                   :bits.next/http-port]))

;;; ----------------------------------------------------------------------------
;;; Datahike

(s/def :bits.datahike/jdbc-url
  (s/and string? #(str/starts-with? % "jdbc:")))

(s/def :bits.datahike/backend keyword?)
(s/def :bits.datahike/id uuid?)
(s/def :bits.datahike/dbtype string?)
(s/def :bits.datahike/host string?)
(s/def :bits.datahike/port pos-int?)
(s/def :bits.datahike/dbname string?)
(s/def :bits.datahike/user string?)
(s/def :bits.datahike/password string?)
(s/def :bits.datahike/table string?)

(s/def :bits.datahike/store
  (s/keys :req-un [:bits.datahike/backend]
          :opt-un [:bits.datahike/id
                   :bits.datahike/dbtype
                   :bits.datahike/host
                   :bits.datahike/port
                   :bits.datahike/dbname
                   :bits.datahike/user
                   :bits.datahike/password
                   :bits.datahike/table]))

(s/def :bits.datahike/config
  (s/keys :req-un [:bits.datahike/store]))

;;; ----------------------------------------------------------------------------
;;; Postgres

(s/def :bits.postgres/config
  (s/keys))

;;; ----------------------------------------------------------------------------
;;; System

;; Rename the keys as we're using unqualified keys to configure our system's
;; components.
(s/def :bits.system/buster :bits.assets/config)
(s/def :bits.system/service :bits.next/config)

(s/def :bits.system/config
  (s/keys :req-un [:bits.system/buster
                   :bits.system/service]))
