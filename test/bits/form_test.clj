(ns bits.form-test
  (:require
   [bits.form :as sut]
   [bits.morph :as morph]
   [bits.test.app :as t]
   [bits.test.browser :as browser]
   [bits.ui :as ui]
   [clojure.test :refer [are deftest is]]
   [matcher-combinators.test :refer [match?]]))

;;; ----------------------------------------------------------------------------
;;; Validate

(def Email
  [:and
   [:string {:min 1}]
   [:re {:error/message "Enter a valid email address"}
    #"^[^\s@]+@[^\s@]+\.[^\s@]+$"]])

(def Password
  [:string {:min 8 :error/message "At least 8 characters"}])

(def schema
  {:email    Email
   :password Password})

(deftest wrap-form-params-normalizes-params
  (let [handler  (fn [req] req)
        wrapped  (sut/wrap-form-params handler)
        request  {:form-params {"email"            "test@example.com"
                                "_unused_password" ""
                                "_target"          "email"
                                "submit"           "true"
                                "action"           "auth/login"
                                "csrf"             "token"}}
        result   (wrapped request)
        form     (::sut/raw result)]
    (is (match? {"email" "test@example.com" "password" ""} (:form-params result)))
    (is (= #{:password} (::sut/pristine form)))
    (is (= :email (::sut/target form)))
    (is (true? (::sut/submitted? form)))))

(deftest validate-form
  (are [expected form]
       (match? expected (sut/validate-form schema form))

    {:email {:status :bits.form/pristine}}
    {::sut/values {:email ""} ::sut/pristine #{:email} ::sut/submitted? false}

    {:email {:status :bits.form/pristine :value "test@example.com"}}
    {::sut/values {:email "test@example.com"} ::sut/pristine #{} ::sut/submitted? false}

    {:email {:status :bits.form/advisory :message string? :used true}}
    {::sut/values {:email "invalid"} ::sut/pristine #{} ::sut/submitted? false}

    {:email {:status :bits.form/error :message string? :used true}}
    {::sut/values {:email "invalid"} ::sut/pristine #{} ::sut/submitted? true}

    {:email {:status :bits.form/error :message "Required" :used true}}
    {::sut/values {:email ""} ::sut/pristine #{} ::sut/submitted? true}

    ;; When editing a field after submission, show advisory instead of error
    {:email {:status :bits.form/advisory :message string? :used true}}
    {::sut/values {:email "invalid"} ::sut/pristine #{} ::sut/submitted? true ::sut/target :email}))

(deftest form-status
  (are [expected validation] (= expected (sut/form-status validation))
    :bits.form/pristine {}
    :bits.form/pristine nil
    :bits.form/pristine {:email {:status :bits.form/pristine}}
    :bits.form/error    {:email {:status :bits.form/pristine} :password {:status :bits.form/error}}
    :bits.form/advisory {:email {:status :bits.form/pristine} :password {:status :bits.form/advisory}}
    :bits.form/pristine {:email {:status :bits.form/pristine} :password {:status :bits.form/pristine}}))

;;; ----------------------------------------------------------------------------
;;; E2E

(def ^:private form-schema
  {:string [:string {:min 3}]
   :email  [:re #".+@.+"]})

(def ^:const ^:private action-key ::action)

(defn- form-view
  [request]
  (let [f (sut/build request {:schema form-schema})]
    (sut/form f action-key
      (sut/field f :string {:label "Name" :type "text"})
      (sut/field f :email {:label "Email" :type "text"})
      (sut/submit f))))

(defn- validate-action
  [request]
  (let [raw (::sut/raw request)]
    (when (or (::sut/submitted? raw) (::sut/target raw))
      (morph/respond (form-view request)))))

(defn- system
  []
  (assoc-in (t/system) [:service :modules]
            {:actions {action-key {:handler validate-action}}
             :routes  [["/" (morph/morphable ui/layout form-view)]]}))

(deftest ^:e2e pristine-no-validation
  (t/with-system [{:keys [service]} (system)]
    (browser/with-driver [driver service]
      (browser/goto driver "/")
      (browser/wait-to-fill driver :string "ab")
      (is (= "ab" (browser/value driver :string)))
      (is (not (browser/invalid? driver :string))))))

(deftest ^:e2e blur-shows-advisory
  (t/with-system [{:keys [service]} (system)]
    (browser/with-driver [driver service]
      (browser/goto driver "/")
      (browser/wait-to-fill driver :string "ab")
      (browser/click driver :email)
      (browser/wait-for-form driver)
      (is (= "ab" (browser/value driver :string)))
      (is (browser/described? driver :string)))))

(deftest ^:e2e submit-shows-errors
  (t/with-system [{:keys [service]} (system)]
    (browser/with-driver [driver service]
      (browser/goto driver "/")
      (browser/click driver "button[type='submit']")
      (browser/wait-for-form driver)
      (is (browser/invalid? driver :string))
      (is (browser/invalid? driver :email))
      (is (= "Whoops!" (browser/text driver :submit))))))

(deftest ^:e2e form-reset
  (t/with-system [{:keys [service]} (system)]
    (browser/with-driver [driver service]
      (browser/goto driver "/")
      (browser/wait-to-fill driver :string "abc")
      (browser/wait-to-fill driver :email "test@example.com")
      (browser/wait-for-form driver)
      (is (= "Submit" (browser/text driver :submit)))
      (browser/click driver "button[type='submit']")
      ;; Wait for success state (button text changes)
      (browser/wait-predicate driver #(= "Success!" (browser/text % :submit)))
      (is (= "" (browser/value driver :string)))
      (is (= "" (browser/value driver :email)))
      (is (= "Success!" (browser/text driver :submit))))))

(deftest ^:e2e values-preserved-on-error
  (t/with-system [{:keys [service]} (system)]
    (browser/with-driver [driver service]
      (browser/goto driver "/")
      (browser/wait-to-fill driver :string "ab")
      (browser/wait-to-fill driver :email "test@example.com")
      (browser/click driver "button[type='submit']")
      (browser/wait-for-form driver)
      (is (= "ab" (browser/value driver :string)))
      (is (= "test@example.com" (browser/value driver :email)))
      (is (browser/invalid? driver :string)))))

(deftest ^:e2e no-race-condition
  (t/with-system [{:keys [service]} (system)]
    (browser/with-driver [driver service]
      (browser/goto driver "/")
      (browser/wait-to-fill driver :string "abc")
      (browser/wait-to-fill driver :email "test@example.com")
      ;; Blur email by clicking string, triggering validation
      (browser/click driver :string)
      (browser/wait-for-form driver)
      ;; Both values should be preserved after morph
      (is (= "abc" (browser/value driver :string)))
      (is (= "test@example.com" (browser/value driver :email))))))
