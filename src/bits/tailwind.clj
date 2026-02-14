(ns bits.tailwind
  (:require
   [steffan-westcott.clj-otel.api.trace.span :as span]
   [winnow.api :as winnow]))

;;; ----------------------------------------------------------------------------
;;; Classes

(defn merge-classes
  [classes]
  {:pre [(vector? classes)]}
  (span/with-span! {:name ::merge-classes}
    (winnow/resolve classes)))
