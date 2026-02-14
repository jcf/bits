(ns bits.tailwind
  (:require
   [steffan-westcott.clj-otel.api.trace.span :as span]
   [winnow.api :as winnow]))

;;; ----------------------------------------------------------------------------
;;; Theme colors

(def ^:private colors
  #{"surface" "surface-raised" "surface-hover"
    "border" "border-subtle"
    "primary" "secondary" "muted"
    "accent" "accent-dim"
    "success"
    "banner-start" "banner-mid"
    "locked-start" "locked-mid" "locked-end"})

;;; ----------------------------------------------------------------------------
;;; Resolver

(def ^:private resolve-classes
  (winnow/make-resolver {:colors colors}))

;;; ----------------------------------------------------------------------------
;;; Classes

(defn- normalize-classes
  [classes]
  (cond
    (nil? classes) []
    (string? classes) [classes]
    :else (vec classes)))

(defn merge-classes
  [classes]
  {:pre [(vector? classes)]}
  (span/with-span! {:name ::merge-classes}
    (resolve-classes classes)))

(defn with-defaults
  [attrs defaults]
  (let [overrides (normalize-classes (:class attrs))]
    (assoc attrs :class (merge-classes (into defaults overrides)))))
