(ns bits.service-test
  (:require
   [bits.datomic :as datomic]
   [bits.test.app :as t]
   [bits.test.fixture :as fixture]
   [clojure.string :as str]
   [clojure.test :refer [deftest is]]
   [datomic.api :as d]
   [matcher-combinators.test]))

;;; ----------------------------------------------------------------------------
;;; Utils

(defn- csp
  [nonce]
  (str/join "; "
            ["default-src 'self'"
             "script-src 'self'"
             "object-src 'none'"
             (format "style-src 'self' 'nonce-%s'" nonce)
             "style-src-attr 'none'"
             "img-src 'self'"]))

(defn- extract-csrf-token
  [response]
  (let [cookies (get-in response [:headers "set-cookie"])]
    (some->> (if (string? cookies) [cookies] cookies)
             (filter #(str/starts-with? % "bits-csrf="))
             first
             (re-find #"bits-csrf=([^;]+)")
             second)))

;;; ----------------------------------------------------------------------------
;;; Secure headers

(deftest secure-headers
  (let [source (constantly (.getBytes "abc"))]
    (t/with-system [{:keys [service]} (t/replace-random-bytes (t/system) source)]
      @(d/transact (datomic/conn (:datomic service)) (fixture/realm-txes))
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
;;; Errors

(deftest unknown-route-returns-404
  (t/with-system [{:keys [service]} (t/system)]
    @(d/transact (datomic/conn (:datomic service)) (fixture/realm-txes))
    (is (match?
         {:status 404}
         (t/request service {:request-method :get :url "/nonexistent"})))))

(deftest invalid-action-returns-400
  (t/with-system [{:keys [service]} (t/system)]
    @(d/transact (datomic/conn (:datomic service)) (fixture/realm-txes))
    (let [client   (t/http-client {:cookie-handler (t/cookie-manager)})
          home     (t/request service {:http-client client :request-method :get :url "/"})
          token    (extract-csrf-token home)
          response (t/request service {:http-client    client
                                       :request-method :post
                                       :url            "/action"
                                       :form-params    {:csrf   token
                                                        :action "not/real"}})]
      (is (match? {:status 400} response)))))

;;; ----------------------------------------------------------------------------
;;; CSRF

(deftest csrf
  (t/with-system [{:keys [service]} (t/system)]
    @(d/transact (datomic/conn (:datomic service)) (fixture/realm-txes))
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
                         :request-method      :post
                         :url         "/action"
                         :form-params {:csrf   token
                                       :action "auth/sign-out"}})))

        (is (match?
             {:status 403}
             (t/request service
                        {:http-client http-client
                         :request-method      :post
                         :url         "/action"
                         :form-params {:action "auth/sign-out"}})))))))

;;; ----------------------------------------------------------------------------
;;; Session

(deftest session-persists-across-requests
  (t/with-system [{:keys [service]} (t/system)]
    @(d/transact (datomic/conn (:datomic service)) (fixture/realm-txes))
    (let [client (t/http-client {:cookie-handler (t/cookie-manager)})
          _first (t/request service {:http-client client :request-method :get :url "/"})
          second (t/request service {:http-client client :request-method :get :url "/"})]
      (is (nil? (get-in second [:headers "set-cookie"]))))))

(deftest session-cookie-attributes
  (t/with-system [{:keys [service]} (t/system)]
    @(d/transact (datomic/conn (:datomic service)) (fixture/realm-txes))
    (let [response (t/request service {:request-method :get :url "/"})
          cookie   (get-in response [:headers "set-cookie"])]
      (is (str/includes? cookie "HttpOnly"))
      (is (str/includes? cookie "SameSite=Lax"))
      (is (str/includes? cookie "Path=/")))))

;;; ----------------------------------------------------------------------------
;;; Auth

(deftest sign-out-clears-session
  (t/with-system [{:keys [service]} (t/system)]
    @(d/transact (datomic/conn (:datomic service)) (fixture/realm-txes))
    (let [client   (t/http-client {:cookie-handler (t/cookie-manager)})
          home     (t/request service {:http-client client :request-method :get :url "/"})
          token    (extract-csrf-token home)
          response (t/request service {:http-client    client
                                       :request-method :post
                                       :url            "/action"
                                       :form-params    {:csrf   token
                                                        :action "auth/sign-out"}})]
      (is (match? {:status 200} response)))))

;;; ----------------------------------------------------------------------------
;;; Realm

(deftest realm
  (t/with-system [{:keys [service]} (t/system)]
    (let [request (t/host {:request-method :get
                           :url            "/"}
                          "foo.bits.page.test")]
      (is (match?
           {:status 404}
           (t/request service request))))))
