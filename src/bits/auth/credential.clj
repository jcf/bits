(ns bits.auth.credential
  (:require
   [bits.datahike :as datahike]
   [steffan-westcott.clj-otel.api.trace.span :as span]))

(defn find-by-email
  "Look up a user and their password hash by email address.
   Returns {:user/id uuid :user/password-hash hash} or nil."
  [database email]
  (span/with-span! {:name ::find-by-email}
    (let [result (datahike/q '[:find ?user-id ?password-hash
                                :in $ ?email
                                :where
                                [?e :email/address ?email]
                                [?e :email/user ?u]
                                [?u :user/id ?user-id]
                                [?u :user/password-hash ?password-hash]]
                              (datahike/db database)
                              email)]
      (when-let [[user-id password-hash] (first result)]
        {:user/id            user-id
         :user/password-hash password-hash}))))

(defn create-user!
  "Create a user with email and hashed password. Returns the user-id."
  [database {:keys [email password-hash]}]
  (span/with-span! {:name ::create-user!}
    (let [user-id (random-uuid)]
      (datahike/transact! database
                          [{:user/id            user-id
                            :user/password-hash password-hash
                            :user/created-at    (java.util.Date.)}
                           {:email/address    email
                            :email/user       [:user/id user-id]
                            :email/preferred? true}])
      user-id)))
