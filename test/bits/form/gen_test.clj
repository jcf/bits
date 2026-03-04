(ns bits.form.gen-test
  (:require
   [bits.test.app :as t]
   [bits.test.browser :as browser]
   [bits.test.form :as test.form]
   [clojure.test :refer [deftest is]]
   [clojure.test.check :as tc]
   [clojure.test.check.generators :as gen]
   [clojure.test.check.properties :as prop]
   [steffan-westcott.clj-otel.api.trace.span :as span]))

;;; ----------------------------------------------------------------------------
;;; Field definitions

(def text-fields [:string :email :number :bio])
(def all-fields [:string :email :number :color :fruit :agree :bio])

;;; ----------------------------------------------------------------------------
;;; Value generators

(def valid-string-gen
  (gen/fmap #(apply str %) (gen/vector gen/char-alpha 3 10)))

(def invalid-string-gen
  (gen/fmap #(apply str %) (gen/vector gen/char-alpha 1 2)))

(def valid-email-gen
  (gen/fmap (fn [[local domain]]
              (str local "@" domain ".com"))
            (gen/tuple (gen/fmap #(apply str %) (gen/vector gen/char-alpha 3 8))
                       (gen/fmap #(apply str %) (gen/vector gen/char-alpha 3 8)))))

(def invalid-email-gen
  (gen/fmap #(apply str %) (gen/vector gen/char-alpha 3 10)))

(def valid-number-gen
  (gen/fmap str (gen/choose 1 999)))

(def invalid-number-gen
  (gen/fmap #(apply str %) (gen/vector gen/char-alpha 1 5)))

(def valid-bio-gen
  (gen/fmap #(apply str %) (gen/vector gen/char-alpha 10 50)))

(def invalid-bio-gen
  (gen/fmap #(apply str %) (gen/vector gen/char-alpha 1 9)))

(def text-gen
  (gen/one-of [valid-string-gen
               invalid-string-gen
               valid-email-gen
               invalid-email-gen
               valid-number-gen
               invalid-number-gen
               valid-bio-gen
               invalid-bio-gen
               (gen/return "")]))

(def color-gen
  (gen/elements (cons "" test.form/color-values)))

(def fruit-gen
  (gen/elements test.form/fruit-values))

;;; ----------------------------------------------------------------------------
;;; Action generators

(def text-field-gen
  (gen/elements text-fields))

(def focus-action-gen
  (gen/fmap (fn [f] [:focus f]) text-field-gen))

(def type-action-gen
  (gen/fmap (fn [[f t]] [:type f t]) (gen/tuple text-field-gen text-gen)))

(def clear-action-gen
  (gen/fmap (fn [f] [:clear f]) text-field-gen))

(def select-action-gen
  (gen/fmap (fn [v] [:select :color v]) color-gen))

(def radio-action-gen
  (gen/fmap (fn [v] [:radio :fruit v]) fruit-gen))

(def toggle-action-gen
  (gen/return [:toggle :agree]))

(def tab-action-gen
  (gen/return [:tab]))

(def submit-action-gen
  (gen/return [:click-submit]))

(def enter-action-gen
  (gen/fmap (fn [f] [:enter f]) text-field-gen))

(def debounce-action-gen
  (gen/return [:debounce]))

(def action-gen
  (gen/frequency
   [[3 focus-action-gen]
    [5 type-action-gen]
    [2 clear-action-gen]
    [2 select-action-gen]
    [2 radio-action-gen]
    [2 toggle-action-gen]
    [2 tab-action-gen]
    [2 submit-action-gen]
    [1 enter-action-gen]
    [1 debounce-action-gen]]))

(def action-sequence-gen
  (gen/vector action-gen 5 20))

;;; ----------------------------------------------------------------------------
;;; Action execution

(defmulti execute-action (fn [_driver action] (first action)))

(defmethod execute-action :focus
  [driver [_ field]]
  (browser/click driver field))

(defmethod execute-action :type
  [driver [_ field text]]
  (browser/click driver field)
  (browser/fill driver field text))

(defmethod execute-action :clear
  [driver [_ field]]
  (browser/click driver field)
  (browser/clear driver field))

(defmethod execute-action :select
  [driver [_ field value]]
  (browser/select-option driver field value))

(defmethod execute-action :radio
  [driver [_ name value]]
  (browser/select-radio driver name value))

(defmethod execute-action :toggle
  [driver [_ field]]
  (browser/toggle driver field))

(defmethod execute-action :tab
  [driver _]
  (browser/press-key driver :tab))

(defmethod execute-action :click-submit
  [driver _]
  (browser/click driver :submit))

(defmethod execute-action :enter
  [driver [_ field]]
  (browser/click driver field)
  (browser/press-key driver :enter))

(defmethod execute-action :debounce
  [_driver _]
  ;; Wait for client-side debounce timer (300ms) to fire.
  ;; See bits.js input handler and the decision record on time coupling.
  (Thread/sleep 310))

;;; ----------------------------------------------------------------------------
;;; Invariant checks

(defn form-not-stuck?
  [driver]
  (browser/wait-for-form driver)
  (nil? (browser/attr driver {:css "form"} "aria-busy")))

(defn aria-invalid-consistent?
  [driver]
  (every? (fn [field]
            (let [invalid? (browser/invalid? driver field)
                  described? (browser/described? driver field)]
              (or (not invalid?) described?)))
          all-fields))

(defn describedby-refs-exist?
  [driver]
  (every? (fn [field]
            (if-let [describedby (browser/attr driver field "aria-describedby")]
              (browser/exists? driver (str "#" describedby))
              true))
          all-fields))

(defn check-invariants
  [driver]
  (span/with-span! {:name ::check-invariants}
    (and (form-not-stuck? driver)
         (aria-invalid-consistent? driver)
         (describedby-refs-exist? driver))))

;;; ----------------------------------------------------------------------------
;;; Property test

(defn form-chaos-property
  [driver]
  (prop/for-all [actions action-sequence-gen]
    (browser/goto driver "/")
    (browser/wait-visible driver :string)
    (doseq [action actions]
      (try
        (span/with-span! {:name       ::execute-action
                          :attributes {"test.action" (pr-str action)}}
          (execute-action driver action))
        (catch Exception _)))
    (check-invariants driver)))

(deftest ^:e2e ^:generative form-handles-chaos
  (t/with-system [{:keys [browser]} (test.form/system)]
    (browser/with-driver [driver browser]
      (let [result (tc/quick-check 25 (form-chaos-property driver))]
        (is (:pass? result)
            (str "Failed: " (get-in result [:shrunk :smallest])))))))
