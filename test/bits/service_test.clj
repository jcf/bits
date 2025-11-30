(ns bits.service-test
  (:require
   [bits.service :as sut]
   [bits.test.app :as test.app]
   [bits.test.service :as t]
   [clojure.string :as str]
   [clojure.test :refer [deftest is testing]]
   [matcher-combinators.test]
   [medley.core :as medley]
   [ring.util.response :as response]))

;;; ----------------------------------------------------------------------------
;;; CORS

(deftest cors
  (let [allowed "http://example.com"]
    (test.app/with-system [{:keys [service]} (test.app/replace-allowed-origins
                                              (test.app/system) #{allowed})]
      (testing "get request"
        (testing "with an allowed origin"
          (let [request  {:request-method :get
                          :url            "/"
                          :headers        {"origin" allowed}}
                response (t/request service request)]
            (is (match? {:status  200
                         :headers {"access-control-allow-methods" "GET, OPTIONS"
                                   "access-control-allow-origin"  "http://example.com"
                                   "access-control-max-age"       "7200"}}
                        response))))
        (testing "with an unknown origin"
          (let [request  {:request-method :get
                          :url            "/"
                          :headers        {"origin" "http://other.example.com"}}
                response (t/request service request)]
            (is (match? {:status  200}
                        response)))))
      (testing "preflight request"
        (testing "with an allowed origin"
          (let [request  {:request-method :options
                          :url            "/"
                          :headers        {"origin" allowed}}
                response (t/request service request)]
            (is (match? {:status  200
                         :headers {"access-control-allow-methods" "GET, OPTIONS"
                                   "access-control-allow-origin"  "http://example.com"
                                   "access-control-max-age"       "7200"
                                   "content-type"                 "text/plain"}
                         :body    ""}
                        response))))
        (testing "with an unknown origin"
          (let [request  {:request-method :options
                          :url            "/"
                          :headers        {"origin" "http://other.example.com"}}
                response (t/request service request)]
            (is (match? {:status  200
                         :headers {"content-type" "text/plain"}
                         :body    ""}
                        response))
            (is (= {}
                   (medley/filter-keys #(str/starts-with? (str/lower-case %) "access-control-")
                                       (:headers response))))))))))

;;; ----------------------------------------------------------------------------
;;; Secure headers

(def ^:private csp
  (str/join "; "
            ["default-src 'self'"
             "script-src 'self'"
             "object-src 'none'"
             "style-src 'self'"
             "style-src-attr 'none'"
             "img-src 'self'"]))

(deftest secure-headers
  (test.app/with-system [{:keys [service]} (test.app/system)]
    (let [request  {:request-method :get
                    :url            "/"}
          response (t/request service request)]
      (is (match?
           {:status  200
            :headers {"content-security-policy"           csp
                      "referrer-policy"                   "strict-origin"
                      "server"                            "Bits"
                      "strict-transport-security"         "max-age=31536000; includeSubdomains"
                      "x-content-type-options"            "nosniff"
                      "x-download-options"                "noopen"
                      "x-frame-options"                   "DENY"
                      "x-permitted-cross-domain-policies" "none"
                      "x-xss-protection"                  "1; mode=block"}}
           response)))))
