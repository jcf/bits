(ns bits.session
  (:require
   [bits.crypto :as crypto]
   [bits.postgres :as postgres]
   [bits.postgres.session :as postgres.session]
   [clojure.spec.alpha :as s]
   [com.stuartsierra.component :as component]
   [next.jdbc :as jdbc]
   [ring.middleware.session.store :as session.store]
   [steffan-westcott.clj-otel.api.trace.span :as span]))

;;; ----------------------------------------------------------------------------
;;; Session operations

(defn new-session
  [randomizer]
  {:nonce (crypto/random-nonce randomizer)
   :sid   (crypto/random-sid randomizer)})

(defn get-session
  "Fetch session by sid. Returns nil if not found or expired."
  [connectable sid]
  {:post [(s/valid? (s/nilable ::postgres.session/persisted) %)]}
  (span/with-span! {:name ::get-session}
    (postgres/execute-one! connectable
                           {:select [:sid :user-id :created-at :data]
                            :from   [:sessions]
                            :where  [:and
                                     [:= :sid sid]
                                     [:> :expires-at [:now]]]})))

(defn create-session!
  "Create session, handling race conditions with ON CONFLICT."
  [connectable sid data idle-timeout-days]
  (span/with-span! {:name ::create-session!}
    (postgres/execute-one! connectable
                           {:insert-into :sessions
                            :values      [{:sid        sid
                                           :data       [:lift data]
                                           :expires-at [:+ [:now]
                                                        [:raw (str "INTERVAL '" idle-timeout-days " days'")]]}]
                            :on-conflict [:sid]
                            :do-nothing  true
                            :returning   [:sid :user-id :created-at :data]})))

(defn touch-session!
  "Update accessed_at and extend expires_at."
  [connectable sid idle-timeout-days]
  (span/with-span! {:name ::touch-session!}
    (postgres/execute-one! connectable
                           {:update :sessions
                            :set    {:accessed-at [:now]
                                     :expires-at  [:+ [:now]
                                                   [:raw (str "INTERVAL '" idle-timeout-days " days'")]]}
                            :where  [:= :sid sid]})))

(defn update-session!
  "Update session data and extend expiry."
  [connectable sid data idle-timeout-days]
  (span/with-span! {:name ::update-session!}
    (postgres/execute-one! connectable
                           {:update :sessions
                            :set    {:data        [:lift data]
                                     :accessed-at [:now]
                                     :expires-at  [:+ [:now]
                                                   [:raw (str "INTERVAL '" idle-timeout-days " days'")]]}
                            :where  [:= :sid sid]})))

(defn rotate-session!
  "Create new session with user-id, delete old session. Returns new sid.
   Prevents session fixation attacks. Runs in a transaction."
  [postgres old-sid user-id idle-timeout-days]
  (span/with-span! {:name ::rotate-session!}
    (let [new-sid (crypto/random-sid (:randomizer postgres))]
      (jdbc/with-transaction [tx (:datasource postgres)]
        (postgres/execute-one! tx
                               {:insert-into :sessions
                                :values      [{:sid        new-sid
                                               :user-id    user-id
                                               :expires-at [:+ [:now]
                                                            [:raw (str "INTERVAL '" idle-timeout-days " days'")]]}]})
        (postgres/execute! tx
                           {:delete-from :sessions
                            :where       [:= :sid old-sid]}))
      new-sid)))

(defn clear-user!
  "Clear user from session (sign-out without full session rotation)."
  [connectable sid idle-timeout-days]
  (span/with-span! {:name ::clear-user!}
    (postgres/execute-one! connectable
                           {:update :sessions
                            :set    {:user-id     nil
                                     :accessed-at [:now]
                                     :expires-at  [:+ [:now]
                                                   [:raw (str "INTERVAL '" idle-timeout-days " days'")]]}
                            :where  [:= :sid sid]})))

(defn delete-session!
  "Delete a session by sid."
  [connectable sid]
  (span/with-span! {:name ::delete-session!}
    (postgres/execute! connectable
                       {:delete-from :sessions
                        :where       [:= :sid sid]})))

(defn delete-expired-sessions!
  "Delete all expired sessions. Returns number of rows deleted."
  [connectable]
  (span/with-span! {:name ::delete-expired-sessions!}
    (let [[{:keys [next.jdbc/update-count]}]
          (postgres/execute! connectable
                             {:delete-from :sessions
                              :where       [:<= :expires-at [:now]]})]
      (or update-count 0))))

;;; ----------------------------------------------------------------------------
;;; Component (implements Ring SessionStore)

(defrecord SessionStore [idle-timeout-days
                         postgres]
  component/Lifecycle
  (start [this]
    (span/with-span! {:name ::start-session-store}
      this))
  (stop [this]
    (span/with-span! {:name ::stop-session-store}
      this))

  session.store/SessionStore
  (read-session [_ sid]
    (when-let [session (some->> sid (get-session postgres))]
      (assoc (::postgres.session/data session)
             :sid     sid
             :user/id (::postgres.session/user-id session))))

  (write-session [_ sid data]
    ;; If data contains :sid different from current, this is a session
    ;; rotation (e.g. post-authentication). Use the new sid — rotate-session!
    ;; already handled the old session deletion.
    (let [new-sid       (:sid data)
          effective-sid (cond
                          (and new-sid (not= new-sid sid)) new-sid
                          (some? sid)                      sid
                          :else                            (crypto/random-sid (:randomizer postgres)))]
      (if (get-session postgres effective-sid)
        (update-session! postgres effective-sid data idle-timeout-days)
        (create-session! postgres effective-sid data idle-timeout-days))
      effective-sid))

  (delete-session [_ sid]
    (some-> sid (->> (delete-session! postgres)))
    nil))

(defn make-session-store
  [{:keys [idle-timeout-days] :or {idle-timeout-days 30}}]
  (map->SessionStore {:idle-timeout-days idle-timeout-days}))
