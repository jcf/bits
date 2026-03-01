(ns bits.form-test
  (:require
   [bits.form :as form]
   [clojure.test :refer [are deftest is]]
   [matcher-combinators.test :refer [match?]]))

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
        wrapped  (form/wrap-form-params handler)
        request  {:form-params {"email"            "test@example.com"
                                "_unused_password" ""
                                "_target"          "email"
                                "submit"           "true"
                                "action"           "auth/login"
                                "csrf"             "token"}}
        result   (wrapped request)
        form     (::form/form result)]
    (is (match? {"email" "test@example.com" "password" ""} (:form-params result)))
    (is (= #{:password} (::form/pristine form)))
    (is (= :email (::form/target form)))
    (is (true? (::form/submitted? form)))))

(deftest validate-form
  (are [expected form]
       (match? expected (form/validate-form schema form))

    {:email {:status :bits.form/pristine}}
    {::form/values {:email ""} ::form/pristine #{:email} ::form/submitted? false}

    {:email {:status :bits.form/pristine :value "test@example.com" :used true}}
    {::form/values {:email "test@example.com"} ::form/pristine #{} ::form/submitted? false}

    {:email {:status :bits.form/advisory :message string? :used true}}
    {::form/values {:email "invalid"} ::form/pristine #{} ::form/submitted? false}

    {:email {:status :bits.form/error :message string? :used true}}
    {::form/values {:email "invalid"} ::form/pristine #{} ::form/submitted? true}

    {:email {:status :bits.form/error :message "Required" :used true}}
    {::form/values {:email ""} ::form/pristine #{} ::form/submitted? true}

    ;; When editing a field after submission, show advisory instead of error
    {:email {:status :bits.form/advisory :message string? :used true}}
    {::form/values {:email "invalid"} ::form/pristine #{} ::form/submitted? true ::form/target :email}))

(deftest form-status
  (are [expected validation] (= expected (form/form-status validation))
    :bits.form/pristine {}
    :bits.form/pristine nil
    :bits.form/pristine {:email {:status :bits.form/pristine}}
    :bits.form/error    {:email {:status :bits.form/pristine} :password {:status :bits.form/error}}
    :bits.form/advisory {:email {:status :bits.form/pristine} :password {:status :bits.form/advisory}}
    :bits.form/pristine {:email {:status :bits.form/pristine} :password {:status :bits.form/pristine}}))
