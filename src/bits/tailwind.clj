(ns bits.tailwind
  (:require
   [clojure.string :as str]
   [clojure.walk :as walk]))

;;; ----------------------------------------------------------------------------
;;; Elements

(def html
  ["antialiased"
   "min-h-screen"])

(def body
  ["flex"
   "flex-col"
   "min-h-screen"
   "bg-neutral-50"
   "dark:bg-neutral-900"])

(def footer
  [])

;;; ----------------------------------------------------------------------------
;;; Classes

(defn- normalize-class
  [x]
  (cond
    (vector? x) x
    (list? x)   (vec x)
    (string? x) (when-not (str/blank? x)
                  (str/split x #"\s+"))))

(defn merge-classes
  [& xs]
  (into []
        (comp (mapcat normalize-class) (remove nil?) (distinct))
        xs))

;;; ----------------------------------------------------------------------------
;;; Component

(defn- expand-template
  [template content-map]
  (walk/postwalk
   (fn [x]
     (if (and (keyword? x) (contains? content-map x))
       (get content-map x)
       x))
   template))

(defn normalize-hiccup
  [args]
  (if (map? (first args))
    args
    (cons {} args)))

(defn- component
  [template & args]
  (let [[opts & children] (normalize-hiccup args)
        tag               (or (:as opts) (:tag template))
        classes           (merge-classes (:classes template) (:class opts))
        attrs             (-> opts
                              (dissoc :as :class)
                              (assoc :class classes))]

    (cond
      (= template :children)
      (into [tag attrs] children)

      (and (vector? template) (= 1 (count children)))
      (expand-template [tag attrs template] {:content (first children)})

      (fn? template)
      (template tag attrs children)

      (map? template)
      (let [slots     (zipmap (:slots template) children)
            structure (expand-template (:structure template) slots)]
        [tag attrs structure])

      :else
      (into [tag attrs] children))))

;;; ----------------------------------------------------------------------------
;;; Templates

(def ^:private templates
  {:button
   {:tag      :button
    :classes  "px-4 py-2 font-semibold rounded-lg shadow-md focus:outline-none focus:ring-2"
    :template :children}

   :card
   {:tag      :div
    :classes  "rounded-lg shadow-lg p-6 bg-white"
    :template :children}

   :card-simple
   {:tag      :article
    :classes  "overflow-hidden rounded-lg shadow-lg bg-white"
    :template [:div {:class "p-6"} :content]}

   :card-with-header
   {:tag      :article
    :classes  "overflow-hidden rounded-lg shadow-lg bg-white"
    :template {:slots     [:<header> :<body>]
               :structure [:div
                           [:header {:class "px-6 py-4 bg-gray-50 border-b border-gray-200"}
                            :<header>]
                           [:div {:class "p-6"}
                            :<body>]]}}

   :alert-with-list
   {:tag      :div
    :classes  "rounded-md bg-red-50 p-4 dark:bg-red-500/15 dark:outline dark:outline-red-500/25"
    :template {:slots [:<title> :<list>]
               :structure
               [:div
                {:class
                 "rounded-md bg-red-50 p-4 dark:bg-red-500/15 dark:outline dark:outline-red-500/25"}
                [:div
                 {:class "flex"}
                 [:div
                  {:class "shrink-0"}
                  [:svg
                   {:viewBox     "0 0 20 20",
                    :fill        "currentColor",
                    :data-slot   "icon",
                    :aria-hidden "true",
                    :class       "size-5 text-red-400"}
                   [:path
                    {:d
                     "M10 18a8 8 0 1 0 0-16 8 8 0 0 0 0 16ZM8.28 7.22a.75.75 0 0 0-1.06 1.06L8.94 10l-1.72 1.72a.75.75 0 1 0 1.06 1.06L10 11.06l1.72 1.72a.75.75 0 1 0 1.06-1.06L11.06 10l1.72-1.72a.75.75 0 0 0-1.06-1.06L10 8.94 8.28 7.22Z",
                     :clip-rule "evenodd",
                     :fill-rule "evenodd"}]]]
                 [:div
                  {:class "ml-3"}
                  [:h3
                   {:class "text-sm font-medium text-red-800 dark:text-red-200"}
                   :<title>]
                  [:div
                   {:class "mt-2 text-sm text-red-700 dark:text-red-200/80"}
                   [:ul
                    {:role "list", :class "list-disc space-y-1 pl-5"}
                    (for [item :<list>]
                      [:li item])]]]]]}}

   :container
   {:tag      :div
    :classes  ["mx-auto max-w-7xl sm:px-6 lg:px-8"]
    :template :children}

   :flex-card
   {:tag      :div
    :classes  "flex flex-col rounded-lg shadow-lg bg-white"
    :template (fn [tag attrs children]
                (let [sections         (partition-by #(= :--- %) children)
                      content-sections (remove #(= [:---] %) sections)]
                  (into [tag attrs]
                        (map-indexed
                         (fn [i section]
                           (into [:div {:class (if (zero? i)
                                                 "p-6"
                                                 "p-6 border-t border-gray-200")}]
                                 section))
                         content-sections))))}})

;;; ----------------------------------------------------------------------------
;;; Tags

(def alert-with-list (partial component (:alert-with-list templates)))
(def card (partial component (:card templates)))
(def container (partial component (:container templates)))
