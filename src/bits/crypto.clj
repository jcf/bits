(ns bits.crypto
  (:refer-clojure :exclude [derive])
  (:require
   [bits.cryptex :as cryptex]
   [buddy.core.codecs :as codecs]
   [buddy.core.mac :as mac]
   [buddy.core.nonce :as nonce]
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

;;; ----------------------------------------------------------------------------
;;; Session crypto

(defn random-sid
  "160-bit (20 byte) secure random, URL-safe base64 encoded."
  []
  (-> (nonce/random-bytes 20)
      (codecs/bytes->b64 true)
      codecs/bytes->str))

(defn csrf-token
  "Compute HMAC-SHA256 of data with secret, URL-safe base64 encoded."
  [secret data]
  (-> (mac/hash data {:key secret :alg :hmac+sha256})
      (codecs/bytes->b64 true)
      codecs/bytes->str))

(comment
  (random-sid)
  (csrf-token "secret" "session-id"))
