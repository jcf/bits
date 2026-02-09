(ns bits.service-test
  (:require
   [bits.test.app :as t]
   [clojure.string :as str]
   [clojure.test :refer [deftest is]]
   [matcher-combinators.test]))

(defn csp
  [nonce]
  (str/join "; "
            ["default-src 'self'"
             "script-src 'self'"
             "object-src 'none'"
             (format "style-src 'self' 'nonce-%s'" nonce)
             "style-src-attr 'none'"
             "img-src 'self'"]))

(deftest secure-headers
  (let [source (constantly (.getBytes "abc"))]
    (t/with-system [{:keys [service]} (t/replace-random-bytes (t/system) source)]
      (let [request {:request-method :get
                     :url            "/"}]
        (is (match?
             {"content-security-policy"           csp
              "referrer-policy"                   "strict-origin"
              "server"                            "Bits"
              "strict-transport-security"         "max-age=31536000; includeSubdomains"
              "x-content-type-options"            "nosniff"
              "x-download-options"                "noopen"
              "x-frame-options"                   "DENY"
              "x-permitted-cross-domain-policies" "none"
              "x-xss-protection"                  "1; mode=block"}
             (:headers (t/request service request))))))))
