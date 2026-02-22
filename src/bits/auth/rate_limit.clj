(ns bits.auth.rate-limit
  (:require
   [bits.anomaly :as anom]
   [bits.crypto :as crypto]
   [bits.locale :refer [tru]]
   [bits.postgres :as postgres]
   [bits.spec]
   [clojure.spec.alpha :as s]
   [com.stuartsierra.component :as component]
   [java-time.api :as time]
   [steffan-westcott.clj-otel.api.metrics.instrument :as instrument]
   [steffan-westcott.clj-otel.api.trace.span :as span]))

;;; ----------------------------------------------------------------------------
;;; Recording

(defn record-attempt!
  [limiter tenant-id params]
  {:pre [(contains? limiter :attempt-counter)]}
  (let [{:keys [attempt-counter postgres]} limiter
        {:keys [email ip-address success]} params]
    (span/with-span! {:name ::record-attempt!}
      (instrument/add! attempt-counter
                       {:value      1
                        :attributes {"tenant_id" (str tenant-id)
                                     "success"   (str (boolean success))}})
      (postgres/execute-one! postgres
                             {:insert-into :authentication-attempts
                              :values      [{:tenant-id tenant-id
                                             :email     email
                                             :ip-hash   (crypto/sha256 ip-address)
                                             :success   (boolean success)}]}))))

;;; ----------------------------------------------------------------------------
;;; Checking

(defn- failure-counts
  [limiter source]
  (let [{:keys [email-window-minutes
                ip-window-minutes
                postgres]}      limiter
        {:keys [tenant-id
                email
                ip-hash]}       source
        window-minutes          (max email-window-minutes ip-window-minutes)
        now                     (time/offset-date-time)
        cutoff                  [:- now [:make-interval :mins window-minutes]]]
    (postgres/execute-one!
     postgres
     {:select [[[:sum [:case [:= :email email] [:inline 1] :else [:inline 0]]] :email-failures]
               [[:sum [:case [:= :ip-hash ip-hash] [:inline 1] :else [:inline 0]]] :ip-failures]]
      :from   [:authentication-attempts]
      :where  [:and
               [:= :tenant-id tenant-id]
               [:not :success]
               [:> :attempted-at cutoff]]})))

(defn- record-rate-limit!
  [limiter tenant-id reason]
  {:pre [(contains? limiter :rate-limit-counter)]}
  (instrument/add! (:rate-limit-counter limiter)
                   {:value      1
                    :attributes {"tenant_id" (str tenant-id)
                                 "reason"    (name reason)}}))

(defn check
  [limiter tenant-id params]
  (let [{:keys [email-max-attempts
                email-window-minutes
                ip-max-attempts
                ip-window-minutes]} limiter
        {:keys [email ip-address]}  params
        source                      {:tenant-id tenant-id
                                     :email     email
                                     :ip-hash   (crypto/sha256 ip-address)}]
    (span/with-span! {:name ::check}
      (let [{:keys [email-failures ip-failures]} (failure-counts limiter source)]
        (cond
          (<= email-max-attempts (or email-failures 0))
          (do
            (record-rate-limit! limiter tenant-id ::email)
            (anom/busy {::anom/message        (tru "Too many attempts. Please try again later.")
                        ::reason              ::email
                        ::retry-after-seconds (* email-window-minutes 60)}))

          (<= ip-max-attempts (or ip-failures 0))
          (do
            (record-rate-limit! limiter tenant-id ::ip)
            (anom/busy {::anom/message        (tru "Too many attempts. Please try again later.")
                        ::reason              ::ip
                        ::retry-after-seconds (* ip-window-minutes 60)})))))))

;;; ----------------------------------------------------------------------------
;;; Cleanup

(defn delete-old-attempts!
  [limiter]
  (let [{:keys [postgres]} limiter]
    (span/with-span! {:name ::delete-old-attempts!}
      (let [now (time/offset-date-time)
            [{:keys [next.jdbc/update-count]}]
            (postgres/execute! postgres
                               {:delete-from :authentication-attempts
                                :where       [:< :attempted-at
                                              [:- now [:make-interval :hours 24]]]})]
        (or update-count 0)))))

;;; ----------------------------------------------------------------------------
;;; Component

(defrecord Limiter [email-max-attempts
                    email-window-minutes
                    ip-max-attempts
                    ip-window-minutes
                    postgres
                    ;; Instruments
                    attempt-counter
                    rate-limit-counter]
  component/Lifecycle
  (start [this]
    (assoc this
           :attempt-counter
           (instrument/instrument {:name            "auth.login.attempt"
                                   :instrument-type :counter
                                   :unit            "{attempt}"
                                   :description     "Login attempts"})
           :rate-limit-counter
           (instrument/instrument {:name            "auth.rate_limit.triggered"
                                   :instrument-type :counter
                                   :unit            "{trigger}"
                                   :description     "Rate limit triggers"})))
  (stop [this]
    (assoc this
           :attempt-counter    nil
           :rate-limit-counter nil)))

(defn make-limiter
  [config]
  {:pre [(s/valid? ::config config)]}
  (map->Limiter config))
