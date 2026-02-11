(ns bits.datahike-test
  (:require
   [bits.cryptex :as cryptex]
   [bits.crypto :as crypto]
   [bits.datahike :as sut]
   [bits.test.app :as t]
   [clojure.test :refer [deftest is]]
   [datahike.api :as d]
   [matcher-combinators.test]))

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

;;; ----------------------------------------------------------------------------
;;; Schema

(defn- hash-password
  [keymaster password]
  (crypto/derive keymaster (cryptex/cryptex password)))

(deftest users
  (t/with-system [{:keys [keymaster datahike]} (t/system)]
    (let [user-txes          [{:user/id            (random-uuid)
                               :user/email         "user@example.com"
                               :user/password-hash (hash-password keymaster "password")}]
          {:keys [db-after]} (sut/transact! datahike user-txes)]
      (is (match?
           [{:user/email         "user@example.com"
             :user/password-hash #"^argon2id"}]
           (d/q '[:find [(pull ?e [*]) ...]
                  :where [?e :user/id]]
                db-after))))))

(deftest creators
  (t/with-system [{:keys [datahike]} (t/system)]
    (let [creator-txes
          [{:creator/bio          "Charlie likes treats and special ball."
            :creator/display-name "Charles Montgomery"
            :creator/handle       "charlie"
            :creator/links        [{:link/icon  :link.icon/instagram
                                    :link/label "Instagram"
                                    :link/url   "https://instagram.com/charliecollie"}]}]
          {:keys [db-after]} (sut/transact! datahike creator-txes)]
      (is (match?
           [{:creator/bio          "Charlie likes treats and special ball."
             :creator/display-name "Charles Montgomery"
             :creator/handle       "charlie"
             :creator/links        [{:link/icon  :link.icon/instagram
                                     :link/label "Instagram"
                                     :link/url   "https://instagram.com/charliecollie"}]}]
           (d/q '[:find [(pull ?e [* {:creator/links [*]}]) ...]
                  :where [?e :creator/handle]]
                db-after))))))
