(ns bits.auth.credential)

(def user-by-email-query
  '[:find (pull ?u [:user/id :user/password-hash]) .
    :in $ ?email
    :where
    [?u :user/email ?email]])
