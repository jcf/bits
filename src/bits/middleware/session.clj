(ns bits.middleware.session
  (:require
   [ring.middleware.cookies :as cookies]
   [ring.middleware.session.store :as store]))

(defn- session-options
  [options]
  {:store        (:store options)
   :cookie-name  (or (:cookie-name options)
                     (throw (ex-info "Missing cookie-name?!" options)))
   :cookie-attrs (merge {:path      "/"
                         :http-only true
                         :same-site :lax
                         :secure    true}
                        (:cookie-attrs options))})

(defn- bare-session-request
  [request {:keys [store cookie-name]}]
  (let [tenant-id   (get-in request [:session/realm :tenant/id])
        sid         (get-in request [:cookies cookie-name :value])
        key         {:tenant-id tenant-id :sid sid}
        session     (when (and tenant-id sid)
                      (store/read-session store key))
        session-key (when session key)]
    (merge request {:session     (or session {})
                    :session/key session-key})))

(defn- bare-session-response
  [response {session-key :session/key :as request} {:keys [store cookie-name cookie-attrs]}]
  (let [tenant-id (get-in request [:session/realm :tenant/id])
        new-session-key
        (when (contains? response :session)
          (if-let [session (:session response)]
            (if (:recreate (meta session))
              (do
                (store/delete-session store session-key)
                (store/write-session store {:tenant-id tenant-id :sid nil}
                                     (vary-meta session dissoc :recreate)))
              (store/write-session store (or session-key {:tenant-id tenant-id})
                                   session))
            (when session-key
              (store/delete-session store session-key))))

        new-sid     (:sid new-session-key)
        old-sid     (:sid session-key)
        cookie      {cookie-name (merge cookie-attrs {:value (or new-sid old-sid)})}
        response    (dissoc response :session :session-cookie-attrs)]
    (if (and new-sid (not= old-sid new-sid))
      (assoc response :cookies (merge (:cookies response) cookie))
      response)))

(defn wrap-session
  [handler options]
  (let [options (session-options options)]
    (fn
      ([request]
       (let [request (-> request cookies/cookies-request (bare-session-request options))]
         (-> (handler request)
             (bare-session-response request options))))
      ([request respond raise]
       (let [request (-> request cookies/cookies-request (bare-session-request options))]
         (handler request
                  (fn [response]
                    (respond (bare-session-response response request options)))
                  raise))))))
