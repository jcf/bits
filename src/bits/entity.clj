(ns bits.entity
  (:require
   [clojure.spec.alpha :as s]))

;;; ----------------------------------------------------------------------------
;;; Money

(s/def :money/amount pos-int?)
(s/def :money/currency keyword?)

(s/def :posting/account uuid?)
(s/def :posting/amount pos-int?)
(s/def :posting/direction #{:posting.direction/credit :posting.direction/debit})
