(ns bits.session
  (:require
   [bits.crypto :as crypto]
   [bits.postgres :as postgres]
   [bits.postgres.session :as postgres.session]
   [bits.spec]
   [clojure.spec.alpha :as s]
   [com.stuartsierra.component :as component]
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
  [store sid]
  {:post [(s/valid? (s/nilable ::postgres.session/persisted) %)]}
  (span/with-span! {:name ::get-session}
    (postgres/execute-one! (:postgres store)
                           {:select [:sid :user-id :created-at :data]
                            :from   [:sessions]
                            :where  [:and
                                     [:= :sid sid]
                                     [:> :expires-at [:now]]]})))

(defn create-session!
  "Create session, handling race conditions with ON CONFLICT."
  [store sid data]
  (let [{:keys [postgres idle-timeout-days]} store]
    (span/with-span! {:name ::create-session!}
      (postgres/execute-one! postgres
                             {:insert-into :sessions
                              :values      [{:sid        sid
                                             :data       [:lift data]
                                             :expires-at [:+ [:now]
                                                          [:make-interval :days idle-timeout-days]]}]
                              :on-conflict [:sid]
                              :do-nothing  true
                              :returning   [:sid :user-id :created-at :data]}))))

(defn touch-session!
  "Update accessed_at and extend expires_at."
  [store sid]
  (let [{:keys [postgres idle-timeout-days]} store]
    (span/with-span! {:name ::touch-session!}
      (postgres/execute-one! postgres
                             {:update :sessions
                              :set    {:accessed-at [:now]
                                       :expires-at  [:+ [:now]
                                                     [:make-interval :days idle-timeout-days]]}
                              :where  [:= :sid sid]}))))

(defn update-session!
  "Update session data and extend expiry."
  [store sid data]
  (let [{:keys [postgres idle-timeout-days]} store]
    (span/with-span! {:name ::update-session!}
      (postgres/execute-one! postgres
                             {:update :sessions
                              :set    {:data        [:lift data]
                                       :accessed-at [:now]
                                       :expires-at  [:+ [:now]
                                                     [:make-interval :days idle-timeout-days]]}
                              :where  [:= :sid sid]}))))

(defn rotate-session!
  "Create new session with user-id, delete old session. Returns new sid.
   Prevents session fixation attacks. Runs in a transaction."
  [store old-sid user-id]
  (let [{:keys [postgres randomizer idle-timeout-days]} store
        new-sid (crypto/random-sid randomizer)]
    (span/with-span! {:name ::rotate-session!}
      (jdbc/with-transaction [tx (:datasource postgres)]
        (postgres/execute-one! tx
                               {:insert-into :sessions
                                :values      [{:sid        new-sid
                                               :user-id    user-id
                                               :expires-at [:+ [:now]
                                                            [:make-interval :days idle-timeout-days]]}]})
        (postgres/execute! tx
                           {:delete-from :sessions
                            :where       [:= :sid old-sid]}))
      new-sid)))

(defn clear-user!
  "Clear user from session (sign-out without full session rotation)."
  [store sid]
  (let [{:keys [postgres idle-timeout-days]} store]
    (span/with-span! {:name ::clear-user!}
      (postgres/execute-one! postgres
                             {:update :sessions
                              :set    {:user-id     nil
                                       :accessed-at [:now]
                                       :expires-at  [:+ [:now]
                                                     [:make-interval :days idle-timeout-days]]}
                              :where  [:= :sid sid]}))))

(defn delete-session!
  [store sid]
  (span/with-span! {:name ::delete-session!}
    (postgres/execute! (:postgres store)
                       {:delete-from :sessions
                        :where       [:= :sid sid]})))

(defn delete-expired-sessions!
  "Delete all expired sessions. Returns number of rows deleted."
  [store]
  (span/with-span! {:name ::delete-expired-sessions!}
    (let [[{:keys [next.jdbc/update-count]}]
          (postgres/execute! (:postgres store)
                             {:delete-from :sessions
                              :where       [:<= :expires-at [:now]]})]
      (or update-count 0))))

;;; ----------------------------------------------------------------------------
;;; Component (implements Ring SessionStore)

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
  (read-session [this sid]
    (when-let [session (some->> sid (get-session this))]
      (assoc (::postgres.session/data session)
             :sid     sid
             :user/id (::postgres.session/user-id session))))

  (write-session [this sid data]
    ;; If data contains :sid different from current, this is a session
    ;; rotation (e.g. post-authentication). Use the new sid — rotate-session!
    ;; already handled the old session deletion.
    (let [new-sid       (:sid data)
          effective-sid (cond
                          (and new-sid (not= new-sid sid)) new-sid
                          (some? sid)                      sid
                          :else                            (crypto/random-sid (:randomizer this)))]
      (if (get-session this effective-sid)
        (update-session! this effective-sid data)
        (create-session! this effective-sid data))
      effective-sid))

  (delete-session [this sid]
    (some-> sid (->> (delete-session! this)))
    nil))

(defn make-session-store
  [config]
  {:pre [(s/valid? ::config config)]}
  (map->SessionStore config))
