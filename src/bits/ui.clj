(ns bits.ui
  (:require
   [bits.asset :as asset]
   [bits.form :as form]
   [bits.locale :refer [tru]]
   [bits.middleware :as mw]
   [bits.tailwind :as tw]))

;;; ----------------------------------------------------------------------------
;;; Input classes

(def ^:private input-base
  ["block" "w-full" "px-3" "py-1.5"
   "text-base" "sm:text-sm/6"
   "bg-surface-raised" "text-primary"
   "placeholder:text-muted"
   "outline-1" "-outline-offset-1"
   "outline-border"
   "focus:relative" "focus:outline-2"
   "focus:-outline-offset-2" "focus:outline-accent"])

;;; ----------------------------------------------------------------------------
;;; Inputs

(defn input
  [attrs]
  [:input (tw/with-defaults attrs input-base)])

(defn input-top
  [attrs]
  (input (tw/with-defaults attrs ["rounded-t-md"])))

(defn input-bottom
  [attrs]
  [:div {:class ["-mt-px"]}
   (input (tw/with-defaults attrs ["rounded-b-md"]))])

;;; ----------------------------------------------------------------------------
;;; Button classes

(def ^:private button-primary-base
  ["flex" "w-full" "justify-center"
   "rounded-md" "px-3" "py-1.5"
   "text-sm/6" "font-semibold" "text-surface"
   "bg-accent" "hover:bg-accent-dim"
   "focus-visible:outline-2"
   "focus-visible:outline-offset-2"
   "focus-visible:outline-accent"])

(def ^:private button-secondary-base
  ["rounded-md" "px-3" "py-1.5"
   "text-sm/6" "font-semibold" "text-primary"
   "bg-surface-hover" "hover:bg-surface-raised"])

;;; ----------------------------------------------------------------------------
;;; Buttons

(defn button-primary
  [attrs & children]
  (into [:button (-> attrs
                     (assoc :type "submit")
                     (tw/with-defaults button-primary-base))]
        children))

(defn button-secondary
  [attrs & children]
  (into [:button (-> attrs
                     (assoc :type "submit")
                     (tw/with-defaults button-secondary-base))]
        children))

;;; ----------------------------------------------------------------------------
;;; Alerts

(def ^:private error-icon-path
  (str "M10 18a8 8 0 1 0 0-16 8 8 0 0 0 0 16Z"
       "M8.28 7.22a.75.75 0 0 0-1.06 1.06L8.94 10l-1.72 1.72"
       "a.75.75 0 1 0 1.06 1.06L10 11.06l1.72 1.72"
       "a.75.75 0 1 0 1.06-1.06L11.06 10l1.72-1.72"
       "a.75.75 0 0 0-1.06-1.06L10 8.94 8.28 7.22Z"))

(defn alert-error
  [message]
  ;; TODO: I18n - error presentation
  [:div {:class ["rounded-md" "bg-red-500/15" "p-4" "mb-4"
                 "outline" "outline-red-500/25"]
         :role  "alert"}
   [:div {:class ["flex"]}
    [:div {:class ["shrink-0"]}
     [:svg {:viewBox     "0 0 20 20"
            :fill        "currentColor"
            :class       ["size-5" "text-red-400"]
            :aria-hidden "true"}
      [:path {:d         error-icon-path
              :clip-rule "evenodd"
              :fill-rule "evenodd"}]]]
    [:div {:class ["ml-3"]}
     [:p {:class ["text-sm" "font-medium" "text-red-200"]}
      message]]]])

;;; ----------------------------------------------------------------------------
;;; Navigation

(defn- nav-links
  []
  [["/"          (tru "Explore")]
   ["/counter"   (tru "Counter")]
   ["/cursors"   (tru "Cursors")]
   ["/email"     (tru "Email")]
   ["/redirect"  (tru "Redirect")]])

(defn nav-header
  [request current-path]
  (let [user       (:session/user request)
        link-class (fn [path]
                     (into ["text-sm" "font-medium"]
                           (if (= path current-path)
                             ["text-accent"]
                             ["text-secondary" "hover:text-primary"])))]
    [:header {:class ["flex" "justify-between" "border-b" "border-border-subtle"]}
     [:nav {:class ["flex" "gap-4" "p-4"]}
      (for [[path label] (nav-links)]
        [:a {:href  path
             :class (link-class path)}
         label])]
     [:div {:class ["p-4"]}
      (if (:user/id user)
        (form/action-button :auth/sign-out
          {:class ["text-sm"
                   "font-medium"
                   "text-secondary"
                   "hover:text-primary"
                   "cursor-pointer"]}
          (tru "Sign out"))
        [:a {:href  "/login"
             :class (link-class "/login")}
         (tru "Login")])]]))

;;; ----------------------------------------------------------------------------
;;; Layout

(def ^:private page-center-base
  ["flex-1" "flex" "flex-col" "justify-center" "items-center"])

(defn page-center
  [attrs & children]
  (into [:div (tw/with-defaults attrs page-center-base)]
        children))

;;; ----------------------------------------------------------------------------
;;; Cards

(def ^:private card-base
  ["p-6" "rounded-lg" "shadow" "max-w-sm"
   "bg-surface-raised" "border" "border-border-subtle"])

(defn card
  [attrs & children]
  (into [:div (tw/with-defaults attrs card-base)]
        children))

(defn card-title
  [& children]
  (into [:h3 {:class ["text-lg" "font-semibold" "mb-4" "text-primary"]}]
        children))

;;; ----------------------------------------------------------------------------
;;; Typography

(defn page-title
  [attrs & children]
  (into [:h1 (tw/with-defaults attrs ["text-4xl" "font-bold" "text-primary"])]
        children))

(defn text-muted
  [attrs & children]
  (into [:p (tw/with-defaults attrs ["text-muted"])]
        children))

(defn text-error
  [& children]
  (into [:p {:class ["text-sm" "text-red-400"]}]
        children))

(defn text-success
  [& children]
  (into [:p {:class ["text-sm" "text-success"]}]
        children))

;;; ----------------------------------------------------------------------------
;;; Icon buttons

(def ^:private icon-button-base
  ["rounded-full" "p-2" "text-surface" "shadow-xs"
   "bg-accent" "hover:bg-accent-dim"
   "focus-visible:outline-2" "focus-visible:outline-offset-2" "focus-visible:outline-accent"])

(defn icon-button
  [attrs & children]
  (into [:button (-> attrs
                     (assoc :type "button")
                     (tw/with-defaults icon-button-base))]
        children))

;;; ----------------------------------------------------------------------------
;;; Not Found

(defn not-found-view
  [_request]
  (page-center {}
    (page-title {} (tru "Page not found"))
    (text-muted {:class ["mt-4"]}
      (tru "The page you''re looking for doesn''t exist."))))

;;; ----------------------------------------------------------------------------
;;; Layout

(defn layout
  [request & content]
  (let [title            (get-in request [:bits/page :page/title] "Bits")
        buster           (mw/request->buster request)
        csrf-cookie-name (mw/request->csrf-cookie-name request)
        asset-path       #(asset/asset-path buster %)]
    [:html {:class ["min-h-screen"] :lang "en"}
     [:head
      [:meta {:charset "UTF-8"}]
      [:meta {:name "viewport" :content "width=device-width, initial-scale=1.0"}]
      [:meta {:name "csrf-cookie" :content csrf-cookie-name}]
      [:title title]
      [:link {:rel "icon" :href (asset-path "/favicon.ico") :sizes "any"}]
      [:link {:rel "icon" :type "image/svg+xml" :href (asset-path "/favicon.svg")}]
      [:link {:rel "apple-touch-icon" :href (asset-path "/apple-touch-icon.png")}]
      [:link {:rel "stylesheet" :href (asset-path "/app.css")}]
      [:script {:src (asset-path "/idiomorph@0.7.4.min.js")}]
      [:script {:src (asset-path "/bits.js")}]]
     [:body {:class ["min-h-screen" "bg-surface" "text-primary" "font-sans"]}
      (into [:main#morph {:class ["min-h-screen" "flex" "flex-col"]}] content)]]))
