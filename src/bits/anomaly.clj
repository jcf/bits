(ns bits.anomaly
  (:require
   [clojure.spec.alpha :as s]))

;;; ----------------------------------------------------------------------------
;;; Categories
;;;
;;; Semantic error categories based on Cognitect's anomalies. Map roughly to
;;; HTTP status families but describe caller-actionable conditions.

(def categories
  #{::busy        ; caller can retry (429/503)
    ::conflict    ; state conflict, resolve and retry (409)
    ::fault       ; internal error (500)
    ::forbidden   ; not authorized (403)
    ::incorrect   ; bad input (400)
    ::interrupted ; request cancelled (499)
    ::not-found   ; resource doesn't exist (404)
    ::unavailable ; service down, don't retry yet (503)
    ::unsupported ; not implemented (501)
    })

;;; ----------------------------------------------------------------------------
;;; Specs

(s/def ::category categories)
(s/def ::message string?)
(s/def ::anomaly (s/keys :req [::category] :opt [::message]))

;;; ----------------------------------------------------------------------------
;;; Predicates

(defn anomaly?
  "Returns true if x is an anomaly."
  [x]
  (and (map? x) (contains? x ::category)))

(defn retryable?
  "Returns true if the anomaly suggests retry may succeed."
  [x]
  (contains? #{::busy ::unavailable ::interrupted} (::category x)))

;;; ----------------------------------------------------------------------------
;;; Constructors

(defn busy        [opts] (assoc opts ::category ::busy))
(defn conflict    [opts] (assoc opts ::category ::conflict))
(defn fault       [opts] (assoc opts ::category ::fault))
(defn forbidden   [opts] (assoc opts ::category ::forbidden))
(defn incorrect   [opts] (assoc opts ::category ::incorrect))
(defn interrupted [opts] (assoc opts ::category ::interrupted))
(defn not-found   [opts] (assoc opts ::category ::not-found))
(defn unavailable [opts] (assoc opts ::category ::unavailable))
(defn unsupported [opts] (assoc opts ::category ::unsupported))
