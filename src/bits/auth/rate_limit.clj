(ns bits.auth.rate-limit
  (:require
   [bits.anomaly :as anom]
   [bits.crypto :as crypto]
   [bits.postgres :as postgres]
   [bits.spec]
   [clojure.spec.alpha :as s]
   [java-time.api :as time]
   [steffan-westcott.clj-otel.api.trace.span :as span]))

;;; ----------------------------------------------------------------------------
;;; Recording

(defn record-attempt!
  [limiter tenant-id params]
  (let [{:keys [postgres]}                 limiter
        {:keys [email ip-address success]} params]
    (span/with-span! {:name ::record-attempt!}
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
          (anom/busy {::anom/message        "Too many attempts. Please try again later."
                      ::reason              ::email
                      ::retry-after-seconds (* email-window-minutes 60)})

          (<= ip-max-attempts (or ip-failures 0))
          (anom/busy {::anom/message        "Too many attempts. Please try again later."
                      ::reason              ::ip
                      ::retry-after-seconds (* ip-window-minutes 60)}))))))

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
                    postgres])

(defn make-limiter
  [config]
  {:pre [(s/valid? ::config config)]}
  (map->Limiter config))
