(ns bits.crypto
  (:refer-clojure :exclude [derive])
  (:require
   [bits.cryptex :as cryptex]
   [bits.spec]
   [buddy.core.codecs :as codecs]
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
;;; Keymaster API

(defn derive
  [keymaster cryptex]
  (hashers/derive (cryptex/reveal cryptex) (:argon keymaster)))

(defn verify
  [_keymaster cryptex hash]
  (hashers/verify (cryptex/reveal cryptex) hash))

;;; ----------------------------------------------------------------------------
;;; Keymaster Component

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
