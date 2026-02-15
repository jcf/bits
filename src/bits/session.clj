(ns bits.session
  (:require
   [bits.crypto :as crypto]
   [bits.postgres :as postgres]
   [bits.postgres.session :as postgres.session]
   [bits.spec]
   [clojure.spec.alpha :as s]
   [com.stuartsierra.component :as component]
   [java-time.api :as time]
   [next.jdbc :as jdbc]
   [ring.middleware.session.store :as session.store]
   [steffan-westcott.clj-otel.api.trace.span :as span]))

;;; ----------------------------------------------------------------------------
;;; Session operations

(defn new-session
  [store]
  (let [randomizer (:randomizer store)]
    {:nonce (crypto/random-nonce randomizer)
     :sid   (crypto/random-sid randomizer)}))

(defn get-session
  "Fetch session by sid. Returns nil if not found or expired."
  [store tenant-id sid]
  {:post [(s/valid? (s/nilable ::postgres.session/persisted) %)]}
  (span/with-span! {:name ::get-session}
    (postgres/execute-one! (:postgres store)
                           {:select [:sid-hash :user-id :created-at :data]
                            :from   [:sessions]
                            :where  [:and
                                     [:= :tenant-id tenant-id]
                                     [:= :sid-hash (crypto/sha256 sid)]
                                     [:> :expires-at (time/offset-date-time)]]})))

(defn create-session!
  "Create session, handling race conditions with ON CONFLICT."
  [store tenant-id sid data]
  (let [{:keys [postgres idle-timeout-days]} store
        now (time/offset-date-time)]
    (span/with-span! {:name ::create-session!}
      (postgres/execute-one! postgres
                             {:insert-into :sessions
                              :values      [{:sid-hash   (crypto/sha256 sid)
                                             :tenant-id  tenant-id
                                             :data       [:lift data]
                                             :expires-at [:+ now
                                                          [:make-interval :days idle-timeout-days]]}]
                              :on-conflict [:sid-hash :tenant-id]
                              :do-nothing  true
                              :returning   [:sid-hash :user-id :created-at :data]}))))

(defn touch-session!
  "Update accessed_at and extend expires_at."
  [store tenant-id sid]
  (let [{:keys [postgres idle-timeout-days]} store
        now (time/offset-date-time)]
    (span/with-span! {:name ::touch-session!}
      (postgres/execute-one! postgres
                             {:update :sessions
                              :set    {:accessed-at now
                                       :expires-at  [:+ now
                                                     [:make-interval :days idle-timeout-days]]}
                              :where  [:and
                                       [:= :tenant-id tenant-id]
                                       [:= :sid-hash (crypto/sha256 sid)]]}))))

(defn update-session!
  "Update session data and extend expiry."
  [store tenant-id sid data]
  (let [{:keys [postgres idle-timeout-days]} store
        now (time/offset-date-time)]
    (span/with-span! {:name ::update-session!}
      (postgres/execute-one! postgres
                             {:update :sessions
                              :set    {:data        [:lift data]
                                       :accessed-at now
                                       :expires-at  [:+ now
                                                     [:make-interval :days idle-timeout-days]]}
                              :where  [:and
                                       [:= :tenant-id tenant-id]
                                       [:= :sid-hash (crypto/sha256 sid)]]}))))

(defn rotate-session!
  "Delete old session, create new session with user-id. Returns new sid.
   Prevents session fixation attacks. Runs in a transaction.
   Order is delete-then-insert so partial failure leaves zero sessions (safe)."
  [store tenant-id old-sid user-id]
  (let [{:keys [postgres randomizer idle-timeout-days]} store
        new-sid (crypto/random-sid randomizer)
        now     (time/offset-date-time)]
    (span/with-span! {:name ::rotate-session!}
      (jdbc/with-transaction [tx (:datasource postgres)]
        (postgres/execute! tx
                           {:delete-from :sessions
                            :where       [:and
                                          [:= :tenant-id tenant-id]
                                          [:= :sid-hash (crypto/sha256 old-sid)]]})
        (postgres/execute-one! tx
                               {:insert-into :sessions
                                :values      [{:sid-hash   (crypto/sha256 new-sid)
                                               :tenant-id  tenant-id
                                               :user-id    user-id
                                               :expires-at [:+ now
                                                            [:make-interval :days idle-timeout-days]]}]}))
      new-sid)))

(defn clear-user!
  "Clear user from session (sign-out without full session rotation).
   Does not extend expiry - only clears user and updates accessed_at."
  [store tenant-id sid]
  (let [{:keys [postgres]} store]
    (span/with-span! {:name ::clear-user!}
      (postgres/execute-one! postgres
                             {:update :sessions
                              :set    {:user-id     nil
                                       :accessed-at (time/offset-date-time)}
                              :where  [:and
                                       [:= :tenant-id tenant-id]
                                       [:= :sid-hash (crypto/sha256 sid)]]}))))

(defn delete-session!
  [store tenant-id sid]
  (span/with-span! {:name ::delete-session!}
    (postgres/execute! (:postgres store)
                       {:delete-from :sessions
                        :where       [:and
                                      [:= :tenant-id tenant-id]
                                      [:= :sid-hash (crypto/sha256 sid)]]})))

(defn delete-expired-sessions!
  "Delete all expired sessions globally. Returns number of rows deleted."
  [store]
  (span/with-span! {:name ::delete-expired-sessions!}
    (let [[{:keys [next.jdbc/update-count]}]
          (postgres/execute! (:postgres store)
                             {:delete-from :sessions
                              :where       [:<= :expires-at (time/offset-date-time)]})]
      (or update-count 0))))

;;; ----------------------------------------------------------------------------
;;; Component (implements Ring SessionStore)
;;;
;;; Key is a compound map: {:tenant-id uuid :sid string}
;;; Middleware constructs this from the resolved tenant and cookie.

(defrecord SessionStore [idle-timeout-days
                         postgres
                         randomizer]
  component/Lifecycle
  (start [this]
    (span/with-span! {:name ::start-session-store}
      this))
  (stop [this]
    (span/with-span! {:name ::stop-session-store}
      this))

  session.store/SessionStore
  (read-session [this {:keys [tenant-id sid]}]
    (when (and tenant-id sid)
      (when-let [session (get-session this tenant-id sid)]
        (assoc (::postgres.session/data session)
               :sid     sid
               :user/id (::postgres.session/user-id session)))))

  (write-session [this {:keys [tenant-id sid] :as key} data]
    ;; If data contains :sid different from current, this is a session
    ;; rotation (e.g. post-authentication). Use the new sid — rotate-session!
    ;; already handled the old session deletion.
    (let [new-sid       (:sid data)
          effective-sid (cond
                          (and new-sid (not= new-sid sid)) new-sid
                          (some? sid)                      sid
                          :else                            (crypto/random-sid (:randomizer this)))]
      (if (get-session this tenant-id effective-sid)
        (update-session! this tenant-id effective-sid data)
        (create-session! this tenant-id effective-sid data))
      (assoc key :sid effective-sid)))

  (delete-session [this {:keys [tenant-id sid]}]
    (when (and tenant-id sid)
      (delete-session! this tenant-id sid))
    nil))

(defn make-session-store
  [config]
  {:pre [(s/valid? ::config config)]}
  (map->SessionStore config))
