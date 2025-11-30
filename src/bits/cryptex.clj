(ns bits.cryptex
  (:require
   [clojure.spec.alpha :as s]
   [clojure.spec.gen.alpha :as sgen]
   [steffan-westcott.clj-otel.api.trace.span :as span])
  (:import
   (clojure.lang IDeref)
   (java.security MessageDigest)))

(def ^:const ^:private placeholder
  "#<Cryptex>")

(def ^:const ^:private hash-code
  11)

(defprotocol IReveal
  (reveal [this]))

(defn- secure-compare
  "Constant-time comparison to prevent timing attacks"
  [a b]
  (let [a-bytes (.getBytes (str a) "UTF-8")
        b-bytes (.getBytes (str b) "UTF-8")]
    (MessageDigest/isEqual a-bytes b-bytes)))

(deftype Cryptex [^:unsynchronized-mutable value]
  IReveal
  (reveal [_this]
    (span/with-span! {:name ::reveal}
      value))

  IDeref
  (deref [this]
    (reveal this))

  Object
  (toString [_]
    placeholder)

  (equals [_this other]
    (and (instance? Cryptex other)
         (secure-compare value (.value ^Cryptex other))))

  (hashCode [_]
    hash-code))

(defn cryptex?
  [x]
  (instance? Cryptex x))

(defn clear!
  [^Cryptex cryptex]
  (set! (.value cryptex) nil))

(s/def ::cryptex
  (s/with-gen cryptex?
    (fn [] (sgen/fmap ->Cryptex (sgen/string)))))

(s/fdef cryptex
  :args (s/cat :s string?)
  :ret  ::cryptex)

(defn cryptex
  [s]
  (span/with-span! {:name ::cryptex}
    (->Cryptex s)))

;; Print method that ensures the value is never printed
(defmethod print-method Cryptex
  [_ ^java.io.Writer w]
  (.write w placeholder))

(comment
  (sgen/generate (sgen/fmap #(vector % (reveal %)) (s/gen ::cryptex))))
