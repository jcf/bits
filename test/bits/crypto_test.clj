(ns bits.crypto-test
  (:require
   [bits.crypto :as sut]
   [clojure.test :refer [deftest is testing]]))

;;; ----------------------------------------------------------------------------
;;; CSRF Tokens
;;;
;;; Token generation must be deterministic - same inputs must produce same
;;; token, otherwise valid requests get rejected as CSRF failures.

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
;;;
;;; Session IDs must be unique to prevent session collisions.

(deftest random-sid-uniqueness
  (testing "generates distinct values"
    (let [sids (repeatedly 100 sut/random-sid)]
      (is (= 100 (count (set sids)))))))
