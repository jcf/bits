(ns bits.money
  (:require
   [clojure.spec.alpha :as s])
  (:import
   (java.text NumberFormat)
   (java.util Currency Locale)))

;;; ----------------------------------------------------------------------------
;;; Specs

(defn currency?
  [x]
  (instance? Currency x))

(s/def ::iso currency?)
(s/def ::money (s/keys :req [:money/amount :money/currency ::iso]))

;;; ----------------------------------------------------------------------------
;;; Enrich

(defn enrich
  [m]
  {:pre [(qualified-keyword? (get-in m [:money/currency :db/ident]))]}
  (let [ident (get-in m [:money/currency :db/ident])]
    (assoc m ::iso (Currency/getInstance (name ident)))))

;;; ----------------------------------------------------------------------------
;;; Format

(defn format-price
  [^Locale locale money]
  {:pre [(some? locale) (currency? (::iso money))]}
  (let [currency (::iso money)
        digits   (.getDefaultFractionDigits currency)
        major    (/ (double (:money/amount money)) (Math/pow 10 digits))
        fmt      (doto (NumberFormat/getCurrencyInstance locale)
                   (.setCurrency currency))]
    (.format fmt major)))
