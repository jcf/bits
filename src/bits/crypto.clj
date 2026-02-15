(ns bits.crypto
  (:refer-clojure :exclude [derive])
  (:require
   [bits.cryptex :as cryptex]
   [bits.spec]
   [buddy.core.codecs :as codecs]
   [buddy.core.hash :as hash]
   [buddy.core.mac :as mac]
   [buddy.core.nonce :as nonce]
   [buddy.hashers :as hashers]
   [clojure.spec.alpha :as s]
   [com.stuartsierra.component :as component]
   [steffan-westcott.clj-otel.api.trace.span :as span]))

(def ^:private dummy-password
  "Constant password for timing oracle prevention."
  "constant-time-dummy-password-bits")

;;; ----------------------------------------------------------------------------
;;; Argon

(defn derive
  [keymaster cryptex]
  (hashers/derive (cryptex/reveal cryptex) (:argon keymaster)))

(defn verify
  [_keymaster cryptex hash]
  (hashers/verify (cryptex/reveal cryptex) hash))

;;; ----------------------------------------------------------------------------
;;; Keymaster

(defrecord Keymaster [argon dummy-hash idle-timeout-days]
  component/Lifecycle
  (start [this]
    (span/with-span! {:name ::start-keymaster}
      (assoc this :dummy-hash (derive this (cryptex/cryptex dummy-password)))))
  (stop [this]
    (span/with-span! {:name ::stop-keymaster}
      (assoc this :dummy-hash nil))))

(defn make-keymaster
  [config]
  {:pre [(s/valid? ::config config)]}
  (map->Keymaster config))

(defmethod print-method Keymaster
  [keymaster ^java.io.Writer w]
  (.write w (format "#<Keymaster idle-timeout-days=%d>"
                    (:idle-timeout-days keymaster))))

;;; ----------------------------------------------------------------------------
;;; Randomizer

(defprotocol Randomize
  (random-bytes [this size]))

(defrecord Randomizer []
  Randomize
  (random-bytes [_this size]
    (nonce/random-bytes size)))

(defmethod print-method Randomizer
  [_ ^java.io.Writer w]
  (.write w "#<Randomizer>"))

(defn make-randomizer
  [config]
  (map->Randomizer config))

;;; ----------------------------------------------------------------------------
;;; CSRF token

(defn csrf-token
  [secret data]
  (span/with-span! {:name ::csrf-token}
    (-> (mac/hash data {:key secret :alg :hmac+sha256})
        (codecs/bytes->b64 true)
        codecs/bytes->str)))

;;; ----------------------------------------------------------------------------
;;; Session ID

(defn random-sid
  [randomizer]
  (span/with-span! {:name ::random-sid}
    (-> (random-bytes randomizer 20)
        (codecs/bytes->b64 true)
        codecs/bytes->str)))

;;; ----------------------------------------------------------------------------
;;; Nonce

(defn random-nonce
  [randomizer]
  (span/with-span! {:name ::random-nonce}
    (codecs/bytes->b64-str (random-bytes randomizer 16) true)))

;;; ----------------------------------------------------------------------------
;;; SHA256

(defn sha256
  [s]
  (codecs/bytes->hex (hash/sha256 s)))
