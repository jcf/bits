(ns bits.postgres-test
  (:require
   [bits.postgres :as sut]
   [bits.test.app :as t]
   [clojure.test :refer [deftest is]]
   [honey.sql :as sql]))

;;; ----------------------------------------------------------------------------
;;; URLs

(def ^:private jdbc-url
  "jdbc:postgresql://127.0.0.1:5432/bits_test?user=bits&password=please")

(deftest dbname
  (is (= "bits_test" (sut/dbname jdbc-url))))

(deftest replace-dbname
  (is (= "jdbc:postgresql://127.0.0.1:5432/postgres?user=bits&password=please"
         (sut/replace-dbname jdbc-url "postgres"))))

;;; ----------------------------------------------------------------------------
;;; Sanitize

(deftest strop
  (is (= "\"\"\"foobar\"\"\""
         (sut/strop \" "\"foobar\"" \"))))

;;; ----------------------------------------------------------------------------
;;; Intervals

(deftest make-interval
  (is (= ["make_interval(days => CAST(? AS INTEGER))" 30]
         (sql/format [:make-interval :days 30]))))

;;; ----------------------------------------------------------------------------
;;; Qualify

(deftest qualification
  (t/with-system [{:keys [postgres]} (t/system)]
    (sut/execute! postgres {:insert-into [:sessions]
                            :values      [{:sid-hash  "abc123"
                                           :tenant-id #uuid "00000000-0000-0000-0000-000000000001"}]})
    (is (match?
         {:bits.postgres.session/sid-hash  "abc123"
          :bits.postgres.session/tenant-id uuid?}
         (sut/execute-one! postgres {:select [:*]
                                     :from   [:sessions]
                                     :limit  1})))))
