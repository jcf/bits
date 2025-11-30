(ns bits.crypto
  (:refer-clojure :exclude [derive])
  (:require
   [bits.cryptex :as cryptex]
   [buddy.hashers :as hashers]
   [clojure.spec.alpha :as s]))

(s/def ::update boolean?)
(s/def ::valid  boolean?)

(s/def ::verification
  (s/keys :req-un [::update ::valid]))

(def argon2id
  {:alg         :argon2id
   :iterations  3
   :memory      (* 64 1024)
   :parallelism 1})

(s/fdef derive
  :args (s/cat :cryptex ::cryptex/cryptex)
  :ret  string?)

(defn derive
  [cryptex]
  (hashers/derive (cryptex/reveal cryptex) argon2id))

(s/fdef verify
  :args (s/cat :string string?)
  :ret  ::verification)

(defn verify
  [cryptex hash]
  (hashers/verify (cryptex/reveal cryptex) hash))

(comment
  (let [secret "secret"]
    (verify (cryptex/cryptex secret) (derive (cryptex/cryptex secret)))))
