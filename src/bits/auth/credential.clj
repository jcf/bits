(ns bits.auth.credential)

(def user-by-email-query
  '[:find (pull ?u [:user/id :user/password-hash]) .
    :in $ ?email
    :where
    [?e :email/address ?email]
    [?e :email/user ?u]])
