(ns bits.form
  (:require
   [bits.html :as html]
   [bits.middleware :as mw]
   [bits.string :as string]
   [bits.tailwind :as tw]))

;;; ----------------------------------------------------------------------------
;;; Form generation

(def ^:private default-attrs
  {:method "post"
   :action "/action"
   :class  ["transition-opacity" "inert:opacity-50" "inert:cursor-wait"]})

(defn form
  [request action-kw & body]
  (let [[opts & children] (html/normalize body)
        csrf              (::mw/csrf request)
        attrs             (-> default-attrs
                              (update :class #(tw/merge-classes (into % (:class opts))))
                              (merge (dissoc opts :class)))]
    (into [:form attrs
           [:input {:type  "hidden"
                    :name  "action"
                    :value (string/keyword->string action-kw)}]
           [:input {:type  "hidden"
                    :name  "csrf"
                    :value csrf}]]
          children)))

(defn action-button
  [action-kw & body]
  (let [[opts children] (html/normalize body)]
    (into [:button (assoc opts
                          :type        "button"
                          :data-action (string/keyword->string action-kw))]
          children)))
