(ns bits.module
  (:require
   [bits.anomaly :as anom]
   [bits.data :refer [keyset]]
   [clojure.spec.alpha :as s]
   [medley.core :as medley]))

;;; ----------------------------------------------------------------------------
;;; Specs

(s/def ::actions :bits.morph/actions)
(s/def ::name qualified-keyword?)
(s/def ::routes vector?)

(s/def :bits/module
  (s/keys :req-un [::actions ::name ::routes]))

(s/def :bits/modules
  (s/coll-of :bits/module))

;;; ----------------------------------------------------------------------------
;;; Normalization

(defn normalize-actions
  [actions]
  (medley/map-vals #(cond->> % (fn? %) (hash-map :handler)) actions))

;;; ----------------------------------------------------------------------------
;;; Validation

(defn- index-actions
  [modules]
  (apply merge-with into
         (for [{:keys [actions name]} modules]
           (zipmap (keys actions) (repeat #{name})))))

(defn combine-modules
  [modules]
  (let [indexed (index-actions modules)
        dupes   (into {} (filter (fn [[_ v]] (< 1 (count v)))) indexed)]
    (if (seq dupes)
      (anom/incorrect {::anom/message "Duplicate action keys?!"
                       :duplicates    dupes})
      {:actions (->> modules
                     (into {} (mapcat :actions))
                     normalize-actions)
       :routes  (into [] (mapcat :routes) modules)})))

(defn must-combine!
  [modules]
  (let [combined (combine-modules modules)]
    (if (anom/anomaly? combined)
      (throw (anom/->exception combined))
      combined)))
