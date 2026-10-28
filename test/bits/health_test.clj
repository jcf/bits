(ns bits.health-test
  (:require
   [bits.health :as health]
   [bits.test.app :as t]
   [clojure.test :refer [deftest is]]
   [matcher-combinators.test]))

;;; ----------------------------------------------------------------------------
;;; Report

(defn- component
  [ready]
  (reify health/Health
    (alive? [_] true)
    (ready? [_] (ready))))

(deftest report-names-what-failed
  (let [report (health/report health/ready?
                              {:fine    (component (constantly true))
                               :refused (component (constantly false))
                               :broken  (component #(throw (ex-info "Boom" {})))
                               :hung    (component #(Thread/sleep 60000))})]
    (is (= {:broken :failed :fine :ok :hung :timeout :refused :failed} report))
    (is (match? {:status 503
                 :body   "broken: failed\nfine: ok\nhung: timeout\nrefused: failed\n"}
                (health/response report)))))

;;; ----------------------------------------------------------------------------
;;; Routes

(deftest healthz-answers-without-a-realm
  (t/with-system [{:keys [service]} (t/system)]
    (is (match?
         {:status 200 :body "datomic: ok\npostgres: ok\n"}
         (t/request service {:request-method :get :url "/healthz"})))
    (is (match?
         {:status 200 :body "datomic: ok\npostgres: ok\n"}
         (t/request service {:request-method :get :url "/livez"})))))
