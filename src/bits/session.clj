(ns bits.session
  (:require
   [clojure.spec.alpha :as s]
   [io.pedestal.log :as log]
   [ring.middleware.session.store :as session.store]
   [steffan-westcott.clj-otel.api.trace.span :as span]))

;;; ----------------------------------------------------------------------------
;;; Cookies

;; Cookie header is the string one might pass in an HTTP request. Cookie value
;; is the form-encoded string paired with the cookie name via an equals sign.
;;
;;     Cookie: <cookie-name>=<cookie-value>
;;
;; `<cookie-header>` is the value of the `Cookie` header above, `<cookie-name>=<cookie-value>`.
;;
;; TODO Improve these names.
(s/def ::cookie-header string?)
(s/def ::cookie-name   string?)
(s/def ::cookie-store  #(satisfies? session.store/SessionStore %))
(s/def ::cookie-value  string?)

(comment
  (s/valid?
   (s/keys)
   {::cookie-header "s=abc--def"
    ::cookie-name   "s"
    ::cookie-store  (reify session.store/SessionStore
                      (read-session [_store _key])
                      (write-session [_store _key _data])
                      (delete-session [_store _key]))
    ::cookie-value  "abc--def"}))

;;; ----------------------------------------------------------------------------
;;; Session map
;;;
;;; Materialized by converting a session map containing a user's ID into a map
;;; of user attributes stored in PostgreSQL.
;;;
;;; Could be worth doing away with the encrypted session cookie and pulling the
;;; session from the database directly.

(s/def ::session
  (s/or :anonymous     nil?
        :authenticated (s/keys)))
