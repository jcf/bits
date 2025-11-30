(ns bits.html
  (:require
   [bits.assets :as assets]
   [bits.interceptor :as i]
   [bits.tailwind :as tw]
   [charred.api :as json]
   [clojure.string :as str]
   [hiccup2.core :as hiccup]
   [steffan-westcott.clj-otel.api.trace.span :as span]))

;;; ------------------------------------------------------------------------------------------------------------------
;;; HTMX

(defn- htmx-config
  []
  (json/write-json-str {"allowEval"              false
                        "allowScriptTags"        false
                        "includeIndicatorStyles" false}))

;;; ------------------------------------------------------------------------------------------------------------------
;;; Layout

(defn layout
  [request & content]
  (let [buster     (i/request->buster request)
        asset-path #(assets/asset-path buster %)]
    [:html {:class tw/html :lang "en"}
     [:head
      [:meta {:name "viewport" :content "width=device-width"}]
      [:meta {:name "htmx-config" :content (htmx-config)}]
      [:title "Welcome to Bits"]
      [:link {:rel "icon" :href "data:,"}]
      [:link {:rel "stylesheet" :href (asset-path "/app.css")}]
      [:script {:src "/htmx@2.0.8.min.js"}]]
     (into [:body {:class tw/body}] content)]))

(defn app
  [request & content]
  (layout
   request
   (list
    (tw/container
     {:as :header :class ["mb-4"]}
     [:h1 "Bits"])
    (into [:main {:class "grow"}] content)
    (tw/container
     {:as :footer}
     (tw/card
      [:p "Built by humans."])))))

;;; ------------------------------------------------------------------------------------------------------------------
;;; HTML

(defn html
  [content]
  (span/with-span! {:name ::html}
    (str "<!DOCTYPE html>" (hiccup/html {:mode :html} content))))

(defn htmx
  [content]
  (span/with-span! {:name ::htmx}
    (str (hiccup/html {:mode :html} content))))
