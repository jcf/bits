(ns bits.string
  (:require
   [clojure.string :as str])
  (:import
   (org.apache.commons.lang3 StringUtils)))

(set! *warn-on-reflection* true)

;;; ----------------------------------------------------------------------------
;;; Non-breaking space

(def ^:const nbsp "\u00A0")

;;; ----------------------------------------------------------------------------
;;; Prefix

(defn remove-prefix
  [^String s ^String prefix]
  {:pre [(string? s) (string? prefix)]}
  (StringUtils/removeStart s prefix))

;;; ----------------------------------------------------------------------------
;;; Suffix

(defn remove-suffix
  [^String s ^String prefix]
  {:pre [(string? s) (string? prefix)]}
  (StringUtils/removeEnd s prefix))

;;; ----------------------------------------------------------------------------
;;; Keyword

(defn keyword->string
  [kw]
  (subs (str kw) 1))

;;; ----------------------------------------------------------------------------
;;; Present

(def present?
  (complement str/blank?))

;;; ----------------------------------------------------------------------------
;;; Words

(defn words
  [s]
  (if (str/blank? s)
    #{}
    (into #{} (str/split (str/trim s) #"\s+"))))
