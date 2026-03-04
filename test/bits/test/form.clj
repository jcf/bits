(ns bits.test.form
  (:require
   [bits.form :as form]
   [bits.morph :as morph]
   [bits.test.app :as t]
   [bits.ui :as ui]))

;;; ----------------------------------------------------------------------------
;;; Schema

(def schema
  {:agree  [:= "true"]
   :bio    [:string {:min 10}]
   :color  [:enum "red" "green" "blue" ""]
   :email  [:re #".+@.+"]
   :fruit  [:enum "apple" "banana" "cherry"]
   :number [:and
            [:string {:min 1}]
            [:re #"^\d+$"]]
   :string [:string {:min 3}]})

;;; ----------------------------------------------------------------------------
;;; Data

(def colors
  [["" "Pick a color"]
   ["red" "Red"]
   ["green" "Green"]
   ["blue" "Blue"]])

(def color-values
  (into [] (remove empty?) (map first colors)))

(def fruits
  [{:option-value "apple" :option-label "Apple"}
   {:option-value "banana" :option-label "Banana"}
   {:option-value "cherry" :option-label "Cherry"}])

(def fruit-values
  (into [] (map :option-value) fruits))

(def params
  {:agree  true
   :bio    "This is my bio and it is long enough to pass validation."
   :color  (first color-values)
   :email  "test@example.com"
   :fruit  (first fruit-values)
   :number "25"
   :string "abc"})

;;; ----------------------------------------------------------------------------
;;; Action

(def ^:const action-key ::action)

;;; ----------------------------------------------------------------------------
;;; Views

(defn form-view
  [request]
  (let [f (form/build request {:schema schema})]
    (form/form f action-key
      (form/field f :string {:label "Name" :type "text"})
      (form/field f :email {:label "Email" :type "email"})
      (form/field f :number {:label "Age" :type "text"})
      (form/select f :color {:label "Favorite color"}
                   (for [[value label] colors]
                     [:option {:value value} label]))
      (form/radio-group f :fruit {:label "Favorite fruit"} fruits)
      (form/checkbox f :agree {:label "I agree to the terms"})
      (form/textarea f :bio {:label "Bio" :rows 3})
      (form/submit f))))

(defn validate-action
  [request]
  (let [raw (::form/raw request)]
    (when (or (::form/submitted? raw) (::form/target raw))
      (morph/respond (form-view request)))))

;;; ----------------------------------------------------------------------------
;;; System

(defn system
  []
  (assoc-in (t/system) [:service :modules]
            {:actions {action-key {:handler validate-action}}
             :routes  [["/" (morph/morphable ui/layout form-view)]]}))
