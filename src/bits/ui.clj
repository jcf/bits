(ns bits.ui
  (:require
   [bits.assets :as assets]
   [bits.form :as form]
   [bits.middleware :as mw]
   [bits.tailwind :as tw]))

;; TODO: I18n - user-facing strings need locale-aware source

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
  [:input (update attrs :class #(tw/merge-classes input-base %))])

(defn input-top
  [attrs]
  (input (update attrs :class #(tw/merge-classes ["rounded-t-md"] %))))

(defn input-bottom
  [attrs]
  [:div {:class "-mt-px"}
   (input (update attrs :class #(tw/merge-classes ["rounded-b-md"] %)))])

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
                     (update :class #(tw/merge-classes button-primary-base %)))]
        children))

(defn button-secondary
  [attrs & children]
  (into [:button (-> attrs
                     (assoc :type "submit")
                     (update :class #(tw/merge-classes button-secondary-base %)))]
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
  [:div {:class ["rounded-md" "bg-red-50" "p-4" "mb-4"
                 "dark:bg-red-500/15"
                 "dark:outline" "dark:outline-red-500/25"]
         :role  "alert"}
   [:div {:class "flex"}
    [:div {:class "shrink-0"}
     [:svg {:viewBox     "0 0 20 20"
            :fill        "currentColor"
            :class       "size-5 text-red-400"
            :aria-hidden "true"}
      [:path {:d         error-icon-path
              :clip-rule "evenodd"
              :fill-rule "evenodd"}]]]
    [:div {:class "ml-3"}
     [:p {:class ["text-sm" "font-medium"
                  "text-red-800" "dark:text-red-200"]}
      message]]]])

;;; ----------------------------------------------------------------------------
;;; Navigation

(def ^:private nav-links
  [["/explore"   "Explore"]
   ["/"          "Counter"]
   ["/cursors"   "Cursors"]
   ["/email"     "Email"]
   ["/redirect"  "Redirect"]])

(defn nav-header
  [request current-path]
  (let [user (:session/user request)
        link-class
        (fn [path]
          (str "text-sm font-medium "
               (if (= path current-path)
                 "text-accent"
                 "text-secondary hover:text-primary")))]
    [:header {:class "flex justify-between border-b border-border-subtle"}
     [:nav {:class "flex gap-4 p-4"}
      (for [[path label] nav-links]
        [:a {:href  path
             :class (link-class path)}
         label])]
     [:div {:class "p-4"}
      (if (:user/id user)
        (form/action-button :auth/sign-out
          {:class ["text-sm"
                   "font-medium"
                   "text-secondary"
                   "hover:text-primary"
                   "cursor-pointer"]}
          "Sign out")
        [:a {:href  "/login"
             :class (link-class "/login")}
         "Login"])]]))

;;; ----------------------------------------------------------------------------
;;; Layout

(def ^:private page-center-base
  ["flex-1" "flex" "flex-col" "justify-center" "items-center"])

(defn page-center
  [attrs & children]
  (into [:div (update attrs :class #(tw/merge-classes page-center-base %))]
        children))

;;; ----------------------------------------------------------------------------
;;; Cards

(def ^:private card-base
  ["p-6" "rounded-lg" "shadow" "max-w-sm"
   "bg-surface-raised" "border" "border-border-subtle"])

(defn card
  [attrs & children]
  (into [:div (update attrs :class #(tw/merge-classes card-base %))]
        children))

(defn card-title
  [& children]
  (into [:h3 {:class "text-lg font-semibold mb-4 text-primary"}]
        children))

;;; ----------------------------------------------------------------------------
;;; Typography

(defn page-title
  [attrs & children]
  (into [:h1 (update attrs :class #(tw/merge-classes
                                    ["text-4xl" "font-bold" "text-primary"]
                                    %))]
        children))

(defn text-muted
  [attrs & children]
  (into [:p (update attrs :class #(tw/merge-classes
                                   ["text-muted"]
                                   %))]
        children))

(defn text-error
  [& children]
  (into [:p {:class "text-sm text-red-600 dark:text-red-400"}]
        children))

(defn text-success
  [& children]
  (into [:p {:class "text-sm text-success"}]
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
                     (update :class #(tw/merge-classes icon-button-base %)))]
        children))

;;; ----------------------------------------------------------------------------
;;; Layout

(defn layout
  [request & content]
  (let [buster     (mw/request->buster request)
        asset-path #(assets/asset-path buster %)]
    [:html {:class "min-h-screen" :lang "en"}
     [:head
      [:meta {:name "viewport" :content "width=device-width"}]
      [:title "Bits"]
      [:link {:rel "icon" :href "data:,"}]
      [:link {:rel "stylesheet" :href (asset-path "/app.css")}]
      [:script {:src (asset-path "/idiomorph@0.7.4.min.js")}]
      [:script {:src (asset-path "/bits.js")}]]
     [:body {:class "min-h-screen bg-surface text-primary font-sans"}
      (into [:main#morph {:class "min-h-screen flex flex-col"}] content)]]))
