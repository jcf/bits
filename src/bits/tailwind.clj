(ns bits.tailwind
  (:require
   [clojure.string :as str]
   [clojure.walk :as walk]
   [steffan-westcott.clj-otel.api.trace.span :as span]
   [winnow.api :as winnow]))

;;; ----------------------------------------------------------------------------
;;; Classes

(defn- normalize-class
  [x]
  {:post [(vector? %)]}
  (span/with-span! {:name ::normalize-class}
    (cond
      (vector? x) x
      (list? x)   (vec x)
      (string? x) (when-not (str/blank? x)
                    (str/split x #"\s+"))
      :else       [])))

(defn merge-classes
  [& xs]
  (let [v (normalize-class xs)]
    (span/with-span! {:name ::resolve}
      (winnow/resolve v))))
