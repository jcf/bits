(ns bits.ui
  (:require
   [bits.tailwind :as tw]))

;; TODO: I18n - user-facing strings need locale-aware source

;;; ----------------------------------------------------------------------------
;;; Input classes

(def ^:private input-base
  ["block" "w-full" "px-3" "py-1.5"
   "text-base" "sm:text-sm/6"
   "bg-white" "dark:bg-neutral-800"
   "text-neutral-900" "dark:text-neutral-100"
   "placeholder:text-neutral-400"
   "outline-1" "-outline-offset-1"
   "outline-neutral-300" "dark:outline-neutral-600"
   "focus:relative" "focus:outline-2"
   "focus:-outline-offset-2" "focus:outline-indigo-600"])

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
   "text-sm/6" "font-semibold" "text-white"
   "bg-indigo-600" "hover:bg-indigo-500"
   "focus-visible:outline-2"
   "focus-visible:outline-offset-2"
   "focus-visible:outline-indigo-600"])

(def ^:private button-secondary-base
  ["rounded-md" "px-3" "py-1.5"
   "text-sm/6" "font-semibold" "text-white"
   "bg-neutral-600" "hover:bg-neutral-500"])

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

(def nav-links
  [["/"         "Counter"]
   ["/cursors"  "Cursors"]
   ["/email"    "Email"]
   ["/login"    "Login"]
   ["/redirect" "Redirect"]])

(defn nav-header
  [current-path]
  [:nav {:class "flex gap-4 p-4 bg-neutral-100 dark:bg-neutral-800"}
   (for [[path label] nav-links]
     [:a {:href  path
          :class (str "text-sm font-medium "
                      (if (= path current-path)
                        "text-indigo-600 dark:text-indigo-400"
                        "text-neutral-600 dark:text-neutral-400 hover:text-neutral-900"))}
      label])])

;;; ----------------------------------------------------------------------------
;;; Layout

(def ^:private page-center-base
  ["min-h-screen" "flex" "flex-col" "justify-center" "items-center"])

(defn page-center
  [attrs & children]
  (into [:div (update attrs :class #(tw/merge-classes page-center-base %))]
        children))

;;; ----------------------------------------------------------------------------
;;; Cards

(def ^:private card-base
  ["p-6" "rounded-lg" "shadow" "max-w-sm"
   "bg-white" "dark:bg-neutral-900"])

(defn card
  [attrs & children]
  (into [:div (update attrs :class #(tw/merge-classes card-base %))]
        children))

(defn card-title
  [& children]
  (into [:h3 {:class "text-lg font-semibold mb-4 text-neutral-900 dark:text-white"}]
        children))

;;; ----------------------------------------------------------------------------
;;; Typography

(defn page-title
  [attrs & children]
  (into [:h1 (update attrs :class #(tw/merge-classes
                                    ["text-4xl" "font-bold"
                                     "text-neutral-900" "dark:text-neutral-100"]
                                    %))]
        children))

(defn text-muted
  [attrs & children]
  (into [:p (update attrs :class #(tw/merge-classes
                                   ["text-neutral-500" "dark:text-neutral-400"]
                                   %))]
        children))

(defn text-error
  [& children]
  (into [:p {:class "text-sm text-red-600 dark:text-red-400"}]
        children))

(defn text-success
  [& children]
  (into [:p {:class "text-sm text-green-600 dark:text-green-400"}]
        children))

;;; ----------------------------------------------------------------------------
;;; Icon buttons

(def ^:private icon-button-base
  ["rounded-full" "p-2" "text-white" "shadow-xs"
   "bg-indigo-600" "hover:bg-indigo-500"
   "focus-visible:outline-2" "focus-visible:outline-offset-2" "focus-visible:outline-indigo-600"
   "dark:bg-indigo-500" "dark:shadow-none" "dark:hover:bg-indigo-400" "dark:focus-visible:outline-indigo-500"])

(defn icon-button
  [attrs & children]
  (into [:button (-> attrs
                     (assoc :type "button")
                     (update :class #(tw/merge-classes icon-button-base %)))]
        children))
