(ns bits.module
  (:require
   [bits.anomaly :as anom]
   [medley.core :as medley]))

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
