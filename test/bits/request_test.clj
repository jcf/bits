(ns bits.request-test
  (:require
   [bits.request :as sut]
   [clojure.test :refer [are deftest]]
   [matcher-combinators.test]
   [ring.mock.request :as mock]))

;;; ----------------------------------------------------------------------------
;;; Remote address

(deftest remote-addr-test
  (are [request addr] (= addr (sut/remote-addr request))
    {:headers {"x-forwarded-for" "203.0.113.1, 70.41.3.18, 150.172.238.178"}}
    "203.0.113.1"

    {:headers {"x-forwarded-for" "203.0.113.1"}}
    "203.0.113.1"

    {:headers {"x-forwarded-for" "  203.0.113.1  , 70.41.3.18"}}
    "203.0.113.1"

    {:remote-addr "127.0.0.1"}
    "127.0.0.1"

    {}
    nil))

;;; ----------------------------------------------------------------------------
;;; Domain

(deftest domain
  (are [request m] (match? m (sut/domain request))
    (mock/request :get "https://example.com/")
    "example.com"

    (mock/request :get "https://example.com:443/")
    "example.com"

    (-> (mock/request :get "https://example.com:443/")
        (update :headers dissoc "host"))
    "example.com"

    (-> (mock/request :get "https://example.com:443/")
        (mock/header "host" "override.test"))
    "override.test"))
