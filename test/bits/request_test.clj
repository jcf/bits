(ns bits.request-test
  (:require
   [bits.request :as sut]
   [clojure.test :refer [deftest is testing]]))

;;; ----------------------------------------------------------------------------
;;; Remote Address Extraction
;;;
;;; Correct IP extraction affects rate limiting and audit logs.

(deftest remote-addr-from-x-forwarded-for
  (testing "extracts first IP from X-Forwarded-For chain"
    (is (= "203.0.113.1"
           (sut/remote-addr {:headers {"x-forwarded-for" "203.0.113.1, 70.41.3.18, 150.172.238.178"}}))))

  (testing "handles single IP"
    (is (= "203.0.113.1"
           (sut/remote-addr {:headers {"x-forwarded-for" "203.0.113.1"}}))))

  (testing "trims whitespace"
    (is (= "203.0.113.1"
           (sut/remote-addr {:headers {"x-forwarded-for" "  203.0.113.1  , 70.41.3.18"}})))))

(deftest remote-addr-fallback
  (testing "uses :remote-addr when no X-Forwarded-For"
    (is (= "127.0.0.1"
           (sut/remote-addr {:remote-addr "127.0.0.1"}))))

  (testing "returns nil when neither present"
    (is (nil? (sut/remote-addr {})))))
