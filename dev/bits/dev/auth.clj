(ns bits.dev.auth
  (:require
   [bits.auth.credential :as credential]
   [bits.cryptex :as cryptex]
   [bits.crypto :as crypto]
   [bits.datomic :as datomic]
   [bits.postgres :as postgres]
   [bits.reaper :as reaper]
   [com.stuartsierra.component.repl :refer [system]]
   [datomic.api :as d]
   [java-time.api :as time]))

(defn- user-txes
  [email password-hash]
  [{:user/id            (random-uuid)
    :user/email         email
    :user/password-hash password-hash
    :user/created-at    (time/java-date)}])

;;; ----------------------------------------------------------------------------
;;; Sessions

(def ^:private all-sessions-query
  {:select   [:*]
   :from     [:sessions]
   :order-by [[:created-at :asc]]})

;;; ----------------------------------------------------------------------------
;;; Authentication attempts

(def ^:private all-attempts-query
  {:select   [:tenant-id :email :ip-hash :success :attempted-at]
   :from     [:authentication-attempts]
   :order-by [[:attempted-at :desc]]})

(def ^:private attempts-by-email-query
  {:select   [:tenant-id :email :ip-hash :success :attempted-at]
   :from     [:authentication-attempts]
   :where    [:= :email :?email]
   :order-by [[:attempted-at :desc]]})

(def ^:private clear-email-rate-limit-query
  {:delete-from :authentication-attempts
   :where       [:and [:= :email :?email] [:not :success]]})

(def ^:private clear-ip-rate-limit-query
  {:delete-from :authentication-attempts
   :where       [:and [:= :ip-hash :?ip-hash] [:not :success]]})

(def ^:private clear-all-attempts-query
  {:delete-from :authentication-attempts})

(comment
  (crypto/random-sid (:randomizer system))
  (crypto/random-nonce (:randomizer system))

  (let [keymaster (:keymaster system)
        password  "password"
        txes      (user-txes
                   "dev@bits.page" (crypto/derive keymaster (cryptex/cryptex password)))]
    @(d/transact (datomic/conn (:datomic system)) txes))

  (d/q credential/user-by-email-query (datomic/db (:datomic system)) "dev@bits.page")

  ;; All sessions
  (postgres/execute! (:postgres system) all-sessions-query)

  ;; All attempts
  (postgres/execute! (:postgres system) all-attempts-query)

  ;; By email
  (postgres/execute!
   (:postgres system) attempts-by-email-query {:params {:email "dev@bits.page"}})

  ;; Clear email rate limit
  (postgres/execute!
   (:postgres system) clear-email-rate-limit-query {:params {:email "dev@bits.page"}})

  ;; Clear IP rate limit (hash the IP first)
  (postgres/execute!
   (:postgres system) clear-ip-rate-limit-query {:ip-hash (crypto/sha256 "127.0.0.1")})

  ;; Clear all
  (postgres/execute! (:postgres system) clear-all-attempts-query)

  (reaper/purge-sessions! (:reaper system)))
