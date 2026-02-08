(ns bits.auth.rate-limit
  (:require
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
  "Record an authentication attempt. Called for both success and failure."
  [pool {:keys [email ip-address success]}]
  (span/with-span! {:name ::record-attempt!}
    (postgres/execute-one! pool
                           {:insert-into :authentication-attempts
                            :values      [{:email      email
                                           :ip-address [:cast ip-address :inet]
                                           :success    (boolean success)}]})))

;;; ----------------------------------------------------------------------------
;;; Checking

(defn- failed-count
  "Count failed attempts within a window."
  [pool where-clause window-minutes]
  (let [result (postgres/execute-one!
                pool
                {:select [[[:count :*] :n]]
                 :from   [:authentication-attempts]
                 :where  [:and
                          where-clause
                          [:not :success]
                          [:> :attempted-at
                           [:- [:now]
                            [:raw (str "INTERVAL '" window-minutes " minutes'")]]]]})]
    (or (:n result) 0)))

(defn throttled?
  "Check whether authentication should be throttled for this email/IP pair.
   Returns nil if allowed, or a map with :reason and :retry-after-seconds if blocked."
  [pool {:keys [email ip-address]}]
  (span/with-span! {:name ::throttled?}
    (let [email-failures (failed-count pool [:= :email email] email-window-minutes)
          ip-failures    (failed-count pool [:= :ip-address [:cast ip-address :inet]] ip-window-minutes)]
      (cond
        (>= email-failures email-max-attempts)
        {:reason              :email
         :retry-after-seconds (* email-window-minutes 60)}

        (>= ip-failures ip-max-attempts)
        {:reason              :ip
         :retry-after-seconds (* ip-window-minutes 60)}))))

;;; ----------------------------------------------------------------------------
;;; Cleanup

(defn delete-old-attempts!
  "Remove attempts older than 24 hours. Call from a scheduled task."
  [pool]
  (span/with-span! {:name ::delete-old-attempts!}
    (let [[{:keys [next.jdbc/update-count]}]
          (postgres/execute! pool
                             {:delete-from :authentication-attempts
                              :where       [:< :attempted-at
                                            [:- [:now] [:raw "INTERVAL '24 hours'"]]]})]
      (or update-count 0))))
