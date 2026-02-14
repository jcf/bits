(ns ^:e2e bits.auth-test
  (:require
   [bits.auth]
   [bits.datomic :as datomic]
   [bits.test.app :as t]
   [bits.test.browser :as browser]
   [bits.test.fixture :as fixture]
   [clojure.test :refer [deftest is]]))

(deftest login
  (t/with-system [{:keys [service]} (t/system)]
    (datomic/transact! (:datomic service) (fixture/realm-txes))
    (let [email    "bits@example.com"
          password "password"]
      (t/create-user! service email password)
      (browser/with-driver [driver service]
        (browser/goto driver "/")
        (browser/click driver {:tag :a :fn/text "Login"})
        (browser/fill driver :email email)
        (browser/fill driver :password password)
        (browser/click driver "button[type='submit']")
        (browser/wait-to-click driver {:tag :button :fn/has-text "Sign out"})
        (is (= "/" (browser/current-path driver)))))))
