(ns bits.morph-test
  (:require
   [bits.crypto :as crypto]
   [bits.morph :as morph]
   [clojure.core.async :as a]
   [clojure.test :refer [deftest is testing]]
   [matcher-combinators.test :refer [match?]]))

;;; ----------------------------------------------------------------------------
;;; SSE

(deftest sse-event-format
  (testing "SSE events follow the wire protocol"
    (let [event (morph/sse-event "morph" "abc123" "payload")]
      (is (= "event: morph\nid: abc123\ndata: payload\n\n" event))))

  (testing "multi-line data is prefixed per SSE spec"
    (let [event (morph/sse-event "test" "id" "line1\nline2")]
      (is (= "event: test\nid: id\ndata: line1\ndata: line2\n\n" event)))))

(deftest morph-event-uses-content-hash-as-id
  (testing "same content produces same event ID for caching"
    (let [html   "<div>test</div>"
          event1 (morph/morph-event html)
          event2 (morph/morph-event html)]
      (is (= event1 event2))))

  (testing "different content produces different event ID"
    (let [event1 (morph/morph-event "<div>a</div>")
          event2 (morph/morph-event "<div>b</div>")]
      (is (not= event1 event2)))))

;;; ----------------------------------------------------------------------------
;;; CSRF

(defn- make-csrf-handler
  []
  (morph/wrap-csrf
   (fn [_] {:status 200 :body "ok"})
   {:cookie-name "csrf" :secret "test-secret"}))

(deftest csrf-rejects-invalid-token
  (let [handler  (make-csrf-handler)
        response (handler {:request-method :post
                           :session        {:sid "session"}
                           :params         {"csrf" "wrong-token"}})]
    (is (match? {:status 403} response))))

(deftest csrf-rejects-missing-token
  (let [handler  (make-csrf-handler)
        response (handler {:request-method :post
                           :session        {:sid "session"}
                           :params         {}})]
    (is (match? {:status 403} response))))

(deftest csrf-accepts-valid-token
  (let [handler (make-csrf-handler)
        sid     "test-session-id"
        token   (crypto/csrf-token "test-secret" sid)]
    (let [response (handler {:request-method :post
                             :session        {:sid sid}
                             :params         {"csrf" token}})]
      (is (match? {:status 200} response)))))

(deftest csrf-allows-safe-methods-without-token
  (let [handler (make-csrf-handler)]
    (doseq [method [:get :head :options]]
      (let [response (handler {:request-method method :params {}})]
        (is (match? {:status 200} response))))))

(deftest csrf-allows-sse-requests-without-token
  (let [handler  (make-csrf-handler)
        response (handler {:request-method :post
                           :params         {}
                           :headers        {"accept" "text/event-stream"}})]
    (is (match? {:status 200} response))))

;;; ----------------------------------------------------------------------------
;;; Actions

(defn- make-action-handler
  [actions]
  (morph/action-handler (morph/normalize-actions actions)))

(defn- action-request
  [action]
  {:parameters        {:form {:action action}}
   :bits.morph/refresh-ch (a/chan)})

(deftest action-handler-unknown-action-returns-400
  (let [handler  (make-action-handler {})
        response (handler (action-request :nonexistent))]
    (is (match? {:status 400} response))))

(deftest action-handler-redirect-sets-location-header
  (let [handler  (make-action-handler {:go (fn [_] (morph/redirect "/target"))})
        response (handler (action-request :go))]
    (is (match? {:status  200
                 :headers {"location" "/target"}}
                response))))

(deftest action-handler-respond-returns-html
  (let [handler  (make-action-handler {:show (fn [_] (morph/respond [:div "hi"]))})
        response (handler (action-request :show))]
    (is (match? {:status  200
                 :headers {"content-type" "text/html; charset=utf-8"}}
                response))))
