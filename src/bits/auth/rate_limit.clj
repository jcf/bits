(ns bits.auth.rate-limit
  (:require
   [bits.anomaly :as anom]
   [bits.postgres :as postgres]
   [steffan-westcott.clj-otel.api.trace.span :as span]))

;;; ----------------------------------------------------------------------------
;;; OWASP rate limiting thresholds
;;;
;;; Two dimensions: per-email and per-IP.
;;; - Per-email: prevents brute-force against a single account
;;; - Per-IP: prevents credential stuffing across many accounts
;;;
;;; Window and threshold are intentionally conservative. The cost of
;;; locking out a legitimate user is high; the cost of a few extra
;;; attempts against Argon2id is low.

(def ^:private email-window-minutes 15)
(def ^:private email-max-attempts 5)

(def ^:private ip-window-minutes 15)
(def ^:private ip-max-attempts 20)

;;; ----------------------------------------------------------------------------
;;; Recording

(defn record-attempt!
  "Record an authentication attempt."
  [connectable params]
  (let [{:keys [email ip-address success]} params]
    (span/with-span! {:name ::record-attempt!}
      (postgres/execute-one! connectable
                             {:insert-into :authentication-attempts
                              :values      [{:email      email
                                             :ip-address [:cast ip-address :inet]
                                             :success    (boolean success)}]}))))

;;; ----------------------------------------------------------------------------
;;; Checking

(defn- failed-count
  [connectable where-clause window-minutes]
  (let [result (postgres/execute-one!
                connectable
                {:select [[[:count :*] :n]]
                 :from   [:authentication-attempts]
                 :where  [:and
                          where-clause
                          [:not :success]
                          [:> :attempted-at
                           [:- [:now]
                            [:raw (str "INTERVAL '" window-minutes " minutes'")]]]]})]
    (or (:n result) 0)))

(defn check
  "Check if request is rate-limited. Returns nil if OK, anomaly if blocked."
  [connectable params]
  (let [{:keys [email ip-address]} params]
    (span/with-span! {:name ::check}
      (let [email-failures (failed-count connectable [:= :email email] email-window-minutes)
            ip-failures    (failed-count connectable [:= :ip-address [:cast ip-address :inet]] ip-window-minutes)]
        (cond
          (<= email-max-attempts email-failures)
          (anom/busy {::anom/message        "Too many attempts. Please try again later."
                      ::reason              ::email
                      ::retry-after-seconds (* email-window-minutes 60)})

          (<= ip-max-attempts ip-failures)
          (anom/busy {::anom/message        "Too many attempts. Please try again later."
                      ::reason              ::ip
                      ::retry-after-seconds (* ip-window-minutes 60)}))))))

;;; ----------------------------------------------------------------------------
;;; Cleanup

(defn delete-old-attempts!
  "Remove attempts older than 24 hours. Call from a scheduled task."
  [connectable]
  (span/with-span! {:name ::delete-old-attempts!}
    (let [[{:keys [next.jdbc/update-count]}]
          (postgres/execute! connectable
                             {:delete-from :authentication-attempts
                              :where       [:< :attempted-at
                                            [:- [:now] [:raw "INTERVAL '24 hours'"]]]})]
      (or update-count 0))))
