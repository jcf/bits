(ns bits.html
  (:require
   [hiccup2.core :as hiccup]
   [steffan-westcott.clj-otel.api.trace.span :as span]))

;;; ----------------------------------------------------------------------------
;;; HTML

(def raw hiccup/raw)

(defn html
  [content]
  (span/with-span! {:name ::html}
    (str "<!DOCTYPE html>" (hiccup/html {:mode :html} content))))

(defn htmx
  [content]
  (span/with-span! {:name ::htmx}
    (str (hiccup/html {:mode :html} content))))
