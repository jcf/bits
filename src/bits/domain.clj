(ns bits.domain
  (:require
   [clojure.spec.alpha :as s]))

;; TODO Combine with bits.schema.
(s/def :user/id uuid?)
(s/def :user/created-at inst?)
(s/def :user/password-hash string?)
