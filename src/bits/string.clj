(ns bits.string
  (:require
   [clojure.string :as str])
  (:import
   (org.apache.commons.lang3 StringUtils)))

(set! *warn-on-reflection* true)

(defn remove-prefix
  [^String s ^String prefix]
  {:pre [(string? s) (string? prefix)]}
  (StringUtils/removeStart s prefix))

(defn remove-suffix
  [^String s ^String prefix]
  {:pre [(string? s) (string? prefix)]}
  (StringUtils/removeEnd s prefix))

(defn keyword->string
  [kw]
  (subs (str kw) 1))

(def present?
  (complement str/blank?))

(defn words
  [s]
  (if (str/blank? s)
    #{}
    (into #{} (str/split (str/trim s) #"\s+"))))
