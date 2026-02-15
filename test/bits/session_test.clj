(ns bits.session-test
  (:require
   [bits.crypto :as crypto]
   [bits.postgres.session :as postgres.session]
   [bits.session :as sut]
   [bits.test.app :as t]
   [clojure.test :refer [deftest is]]
   [java-time.api :as time]
   [matcher-combinators.test]))

(def ^:private tenant-id
  #uuid "df0c1ec1-1cbe-4c35-a447-057fd22a1239")

(deftest get-session-with-unknown-session-id
  (t/with-system [{:keys [randomizer session-store]} (t/system)]
    (is (nil? (sut/get-session session-store
                               tenant-id
                               (crypto/random-sid randomizer))))))

(deftest get-session-with-a-known-session-id
  (t/with-system [{:keys [session-store]} (t/system)]
    (let [{:keys [sid]
           :as   data} (sut/new-session session-store)
          sid-hash     (crypto/sha256 sid)
          expired      (-> session-store :idle-timeout-days time/days)]
      (sut/create-session! session-store tenant-id sid data)
      (is (match?
           {::postgres.session/created-at inst?,
            ::postgres.session/data       {:nonce string?
                                           ;; FIXME Remove `:sid` from `data`.
                                           :sid   sid}
            ::postgres.session/sid-hash   sid-hash
            ::postgres.session/user-id    nil}
           (sut/get-session session-store tenant-id sid)))
      (time/with-clock (time/mock-clock (time/plus (time/instant) expired))
        (is (nil? (sut/get-session session-store tenant-id sid)))))))

(deftest touch-session-extends-expiry
  (t/with-system [{:keys [session-store]} (t/system)]
    (let [{:keys [sid] :as data} (sut/new-session session-store)
          sid-hash               (crypto/sha256 sid)
          timeout-days           (:idle-timeout-days session-store)
          almost-expired         (time/hours (- (* timeout-days 24) 1))]
      (sut/create-session! session-store tenant-id sid data)
      (time/with-clock (time/mock-clock (time/plus (time/instant) almost-expired))
        (sut/touch-session! session-store tenant-id sid))
      (time/with-clock (time/mock-clock (time/plus (time/instant)
                                                   (time/hours (inc (* timeout-days 24)))))
        (is (match?
             {::postgres.session/sid-hash sid-hash}
             (sut/get-session session-store tenant-id sid)))))))

(deftest delete-expired-sessions-removes-only-expired
  (t/with-system [{:keys [session-store]} (t/system)]
    (let [valid-session   (sut/new-session session-store)
          expired-session (sut/new-session session-store)
          timeout-days    (:idle-timeout-days session-store)]
      (sut/create-session! session-store tenant-id (:sid valid-session) valid-session)
      (sut/create-session! session-store tenant-id (:sid expired-session) expired-session)
      (time/with-clock (time/mock-clock (time/plus (time/instant)
                                                   (time/days (inc timeout-days))))
        (let [deleted (sut/delete-expired-sessions! session-store)]
          (is (= 2 deleted))))
      (is (nil? (sut/get-session session-store tenant-id (:sid valid-session))))
      (is (nil? (sut/get-session session-store tenant-id (:sid expired-session)))))))
