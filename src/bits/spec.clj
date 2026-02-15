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

(s/def :bits.asset/resources (s/coll-of string? :kind set?))

(s/def :bits.asset/config
  (s/keys :req-un [:bits.asset/resources]))

;;; ----------------------------------------------------------------------------
;;; Morph
;;;
;;; These specs live here to avoid cyclic dependencies. bits.morph may require
;;; bits.spec, so bits.spec cannot require bits.morph.

(s/def :bits.morph/handler fn?)
(s/def :bits.morph/params vector?)
(s/def :bits.morph/action-map
  (s/keys :req-un [:bits.morph/handler]
          :opt-un [:bits.morph/params]))
(s/def :bits.morph/action
  (s/or :fn fn? :map :bits.morph/action-map))
(s/def :bits.morph/actions
  (s/map-of qualified-keyword? :bits.morph/action))

;;; ----------------------------------------------------------------------------
;;; Rate limiter

(s/def :bits.auth.rate-limit/email-max-attempts pos-int?)
(s/def :bits.auth.rate-limit/email-window-minutes pos-int?)
(s/def :bits.auth.rate-limit/ip-max-attempts pos-int?)
(s/def :bits.auth.rate-limit/ip-window-minutes pos-int?)

(s/def :bits.auth.rate-limit/config
  (s/keys :req-un [:bits.auth.rate-limit/email-max-attempts
                   :bits.auth.rate-limit/email-window-minutes
                   :bits.auth.rate-limit/ip-max-attempts
                   :bits.auth.rate-limit/ip-window-minutes]))

;;; ----------------------------------------------------------------------------
;;; Service

(s/def :bits.service/actions :bits.morph/actions)
(s/def :bits.service/cookie-name string?)
(s/def :bits.service/csrf-cookie-name string?)
(s/def :bits.service/csrf-secret string?)
(s/def :bits.service/http-host string?)
(s/def :bits.service/http-port (s/or :zero zero? :pos-int pos-int?))
(s/def :bits.service/routes vector?)
(s/def :bits.service/server-name string?)
(s/def :bits.service/sse-reconnect-ms pos-int?)

(s/def :bits.service/config
  (s/keys :req-un [:bits.service/actions
                   :bits.service/cookie-name
                   :bits.service/csrf-cookie-name
                   :bits.service/csrf-secret
                   :bits.service/http-host
                   :bits.service/http-port
                   :bits.service/routes
                   :bits.service/server-name
                   :bits.service/sse-reconnect-ms]))

;;; ----------------------------------------------------------------------------
;;; Datomic

(s/def :bits.datomic/uri string?)

(s/def :bits.datomic/config
  (s/keys :req-un [:bits.datomic/uri]))

;;; ----------------------------------------------------------------------------
;;; Crypto

(s/def :bits.crypto/argon map?)
(s/def :bits.crypto/config
  (s/keys :req-un [:bits.crypto/argon]))

;;; ----------------------------------------------------------------------------
;;; Session

(s/def :bits.session/idle-timeout-days pos-int?)
(s/def :bits.session/config
  (s/keys :req-un [:bits.session/idle-timeout-days]))

;;; ----------------------------------------------------------------------------
;;; Postgres

(s/def :bits.postgres/config
  (s/keys))

;;; ----------------------------------------------------------------------------
;;; System

;; Rename the keys as we're using unqualified keys to configure our system's
;; components.
(s/def :bits.system/buster :bits.asset/config)
(s/def :bits.system/service :bits.service/config)

(s/def :bits.system/config
  (s/keys :req-un [:bits.system/buster
                   :bits.system/service]))
