(ns bits.dev.auth
  (:require
   [bits.auth.credential :as credential]
   [bits.cryptex :as cryptex]
   [bits.crypto :as crypto]
   [bits.datomic :as datomic]
   [bits.postgres :as postgres]
   [bits.reaper :as reaper]
   [com.stuartsierra.component.repl :refer [system]]
   [java-time.api :as time]))

(defn- user-txes
  [email password-hash]
  [{:user/id            (random-uuid)
    :user/email         email
    :user/password-hash password-hash
    :user/created-at    (time/java-date)}])

(comment
  (let [keymaster (:keymaster system)
        password  "password"
        txes      (user-txes
                   "dev@bits.page" (crypto/derive keymaster (cryptex/cryptex password)))]
    (datomic/transact! (:datomic system) txes))

  (datomic/q (:datomic system) credential/user-by-email-query "dev@bits.page")

  (postgres/execute! (:postgres system)
                     {:select   [:*]
                      :from     [:sessions]
                      :order-by [[:created-at :asc]]})

  (reaper/purge-sessions! (:reaper system)))
