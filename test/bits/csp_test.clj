(ns bits.csp-test
  (:require
   [bits.csp :as sut]
   [clojure.test :refer [deftest is]]))

(deftest policy
  (let [m {:default-src    "'self'"
           :img-src        "'self'"
           :object-src     "'none'"
           :script-src     "'self'"
           :style-src      "'self'"
           :style-src-attr "'none'"}]
    (is (= m
           (sut/policy)))
    (is (= (assoc m :style-src "'self' 'nonce-abc'")
           (sut/policy "abc")))))
