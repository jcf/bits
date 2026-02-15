(ns bits.postgres.session
  (:require
   [clojure.spec.alpha :as s]))

(s/def ::created-at inst?)
(s/def ::data map?)
(s/def ::sid-hash string?)
(s/def ::user-id (s/nilable uuid?))

(s/def ::persisted
  (s/keys :req [::data ::sid-hash]
          :opt [::created-at ::data]))
