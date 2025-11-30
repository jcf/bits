(ns bits.spec
  (:require
   [clojure.spec.alpha :as s]
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

(s/def :bits.service/allow-credentials? boolean?)
(s/def :bits.service/allowed-headers (s/coll-of string? :kind set?))
(s/def :bits.service/allowed-origins (s/coll-of string? :kind set?))
(s/def :bits.service/canonical-host string?)
(s/def :bits.service/cookie-name string?)
(s/def :bits.service/cookie-secret bytes?)
(s/def :bits.service/http-host string?)
(s/def :bits.service/http-port (s/or :zero zero? :pos-int pos-int?))
(s/def :bits.service/join? boolean?)
(s/def :bits.service/name string?)
(s/def :bits.service/origin string?)
(s/def :bits.service/server-header string?)

(s/def :bits.service/config
  (s/keys :req-un [:bits.service/allow-credentials?
                   :bits.service/allowed-headers
                   :bits.service/allowed-origins
                   :bits.service/canonical-host
                   :bits.service/cookie-secret
                   :bits.service/http-host
                   :bits.service/http-port
                   :bits.service/join?
                   :bits.service/name
                   :bits.service/origin
                   :bits.service/server-header]
          :opt-un [:bits.service/cookie-name]))

;;; ----------------------------------------------------------------------------
;;; System

;; Rename the keys as we're using unqualified keys to configure our system's
;; components.
(s/def :bits.system/buster :bits.assets/config)
(s/def :bits.system/service :bits.service/config)

(s/def :bits.system/config
  (s/keys :req-un [:bits.system/buster
                   :bits.system/service]))
