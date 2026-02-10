(ns bits.datahike-test
  (:require
   [bits.datahike :as sut]
   [clojure.test :refer [deftest is]]))

;;; ----------------------------------------------------------------------------
;;; JDBC URL

(def ^:private jdbc-url
  "jdbc:postgresql://127.0.0.1:5432/bits_test?user=bits&password=please")

(deftest jdbc-url->store
  (is (= {:backend  :jdbc
          :dbname   "bits_test"
          :dbtype   "postgresql"
          :host     "127.0.0.1"
          :id       #uuid "24c0d1fb-9382-5cbf-b566-059520400471"
          :password "please"
          :port     5432
          :table    "datahike"
          :user     "bits"}
         (sut/jdbc-url->store jdbc-url))))
