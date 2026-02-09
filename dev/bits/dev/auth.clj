(ns bits.dev.auth
  (:require
   [bits.auth.credential :as credential]
   [bits.cryptex :as cryptex]
   [bits.crypto :as crypto]
   [bits.datahike :as datahike]
   [bits.next.reaper :as reaper]
   [bits.postgres :as postgres]
   [com.stuartsierra.component.repl :refer [system]]
   [datahike.core]))

(defn- user-txes
  [email password-hash]
  (let [tempid (datahike.core/tempid :db.part/ignored)]
    [{:db/id              tempid
      :user/id            (random-uuid)
      :user/password-hash password-hash
      :user/created-at    (java.util.Date.)}
     {:email/address    email
      :email/user       tempid
      :email/preferred? true}]))

(comment
  (let [keymaster (:keymaster system)
        password  "password"
        txes      (user-txes
                   "dev@bits.page" (crypto/derive keymaster (cryptex/cryptex password)))]
    (datahike/transact! (:datahike system) txes))

  (datahike/q (:datahike system) credential/user-by-email-query "dev@bits.page")

  (postgres/execute! (:postgres system)
                     {:select   [:*]
                      :from     [:sessions]
                      :order-by [[:created-at :asc]]})

  (reaper/purge-sessions! (:reaper system)))
