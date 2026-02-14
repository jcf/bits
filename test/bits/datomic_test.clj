(ns bits.datomic-test
  (:require
   [bits.cryptex :as cryptex]
   [bits.crypto :as crypto]
   [bits.datomic :as sut]
   [bits.test.app :as t]
   [clojure.test :refer [deftest is]]
   [datomic.api :as d]
   [matcher-combinators.test]))

;;; ----------------------------------------------------------------------------
;;; Schema

(defn- hash-password
  [keymaster password]
  (crypto/derive keymaster (cryptex/cryptex password)))

(deftest users
  (t/with-system [{:keys [keymaster datomic]} (t/system)]
    (let [user-txes          [{:user/id            (random-uuid)
                               :user/email         "user@example.com"
                               :user/password-hash (hash-password keymaster "password")}]
          {:keys [db-after]} (sut/transact! datomic user-txes)]
      (is (match?
           [{:user/email         "user@example.com"
             :user/password-hash #"^argon2id"}]
           (d/q '[:find [(pull ?e [*]) ...]
                  :where [?e :user/id]]
                db-after))))))

(deftest creators
  (t/with-system [{:keys [datomic]} (t/system)]
    (let [creator-txes
          [{:creator/bio          "Charlie likes treats and special ball."
            :creator/display-name "Charles Montgomery"
            :creator/handle       "charlie"
            :creator/links        [{:link/icon  :link.icon/instagram
                                    :link/label "Instagram"
                                    :link/url   "https://instagram.com/charliecollie"}]}]
          {:keys [db-after]} (sut/transact! datomic creator-txes)]
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
