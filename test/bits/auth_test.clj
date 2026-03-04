(ns ^:e2e bits.auth-test
  (:require
   [bits.datomic :as datomic]
   [bits.test.app :as t]
   [bits.test.browser :as browser]
   [bits.test.fixture :as fixture]
   [clojure.test :refer [deftest is]]
   [datomic.api :as d]))

(deftest login
  (t/with-system [{:keys [service browser]} (t/system)]
    @(d/transact (datomic/conn (:datomic service)) (fixture/realm-txes))
    (let [email    "bits@example.com"
          password "password"]
      (t/create-user! service email password)
      (browser/with-driver [driver browser]
        (browser/goto driver "/")
        (browser/click driver {:tag :a :fn/text "Login"})
        (browser/wait-to-fill driver :email email)
        (browser/wait-to-fill driver :password password)
        (browser/click driver "button[type='submit']")
        (browser/wait-to-click driver {:tag :button :fn/has-text "Sign out"})
        (is (= "/" (browser/current-path driver)))))))
