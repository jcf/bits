(ns bits.crypto-test
  (:require
   [bits.crypto :as sut]
   [bits.test.app :as t]
   [clojure.test :refer [deftest is testing]]))

;;; ----------------------------------------------------------------------------
;;; CSRF Tokens

(deftest csrf-token-deterministic
  (testing "same secret and session produce identical tokens"
    (is (= (sut/csrf-token "secret" "session-123")
           (sut/csrf-token "secret" "session-123")))))

(deftest csrf-token-varies-with-inputs
  (testing "different secrets produce different tokens"
    (is (not= (sut/csrf-token "secret-a" "session")
              (sut/csrf-token "secret-b" "session"))))

  (testing "different sessions produce different tokens"
    (is (not= (sut/csrf-token "secret" "session-a")
              (sut/csrf-token "secret" "session-b")))))

;;; ----------------------------------------------------------------------------
;;; Session IDs

(deftest random-sid-uniqueness
  (t/with-system [{:keys [randomizer]} (t/system)]
    (testing "generates distinct values"
      (let [sids (repeatedly 100 #(sut/random-sid randomizer))]
        (is (= 100 (count (set sids))))))))
