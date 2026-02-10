(ns bits.service-test
  (:require
   [bits.test.app :as t]
   [clojure.string :as str]
   [clojure.test :refer [deftest is]]
   [matcher-combinators.test]))

;;; ----------------------------------------------------------------------------
;;; Secure headers

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
             {"content-security-policy"           (csp "YWJj")
              "referrer-policy"                   "strict-origin"
              "server"                            "Bits"
              "strict-transport-security"         "max-age=31536000; includeSubdomains"
              "x-content-type-options"            "nosniff"
              "x-download-options"                "noopen"
              "x-frame-options"                   "DENY"
              "x-permitted-cross-domain-policies" "none"
              "x-xss-protection"                  "1; mode=block"}
             (:headers (t/request service request))))))))

;;; ----------------------------------------------------------------------------
;;; CSRF

(defn- extract-csrf-token
  [response]
  (let [cookies (get-in response [:headers "set-cookie"])]
    (some->> (if (string? cookies) [cookies] cookies)
             (filter #(str/starts-with? % "bits-csrf="))
             first
             (re-find #"bits-csrf=([^;]+)")
             second)))

(deftest csrf
  (t/with-system [{:keys [service]} (t/system)]
    (let [http-client (t/http-client {:cookie-handler (t/cookie-manager)})
          request     {:http-client    http-client
                       :request-method :get
                       :url            "/"}
          response    (t/request service request)
          token       (extract-csrf-token response)]

      (when (is (string? token)
                (str "Token: " (pr-str token)))
        (is (match?
             {:status 200}
             (t/request service
                        {:http-client http-client
                         :method      :post
                         :url         "/action"
                         :form-params {:csrf   token
                                       :action "auth/sign-out"}})))

        (is (match?
             {:status 403}
             (t/request service
                        {:http-client http-client
                         :method      :post
                         :url         "/action"
                         :form-params {:action "auth/sign-out"}})))))))
