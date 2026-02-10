(ns bits.auth
  (:require
   [bits.anomaly :as anom]
   [bits.auth.credential :as credential]
   [bits.auth.rate-limit :as rate-limit]
   [bits.cryptex :as cryptex]
   [bits.crypto :as crypto]
   [bits.datahike :as datahike]
   [bits.form :as form]
   [bits.middleware :as mw]
   [bits.morph :as morph]
   [bits.request :as request]
   [bits.session :as session]
   [bits.ui :as ui]
   [io.pedestal.log :as log]
   [next.jdbc :as jdbc]
   [steffan-westcott.clj-otel.api.trace.span :as span]))

;;; ----------------------------------------------------------------------------
;;; Views

;; TODO: I18n - user-facing strings need locale-aware source

(defn login-view
  [request options]
  (let [{:keys [error]} options]
    (list
     (ui/nav-header request "/login")
     (ui/page-center {:class ["px-6" "py-12" "lg:px-8"]}
                     [:div {:class ["sm:mx-auto" "sm:w-full" "sm:max-w-sm"]}
                      [:h2 {:class ["mt-10" "text-center" "text-2xl/9" "font-bold"
                                    "tracking-tight" "text-neutral-900" "dark:text-white"]}
                       "Sign in to your account"]]
                     [:div {:class ["mt-10" "sm:mx-auto" "sm:w-full" "sm:max-w-sm"]}
                      (when error
                        (ui/alert-error error))
                      (form/form request :auth/login
                        [:div
                         (ui/input-top {:type        "email"
                                        :name        "email"
                                        :placeholder "Email address"
                                        :required    true
                                        :autofocus   true})
                         (ui/input-bottom {:type        "password"
                                           :name        "password"
                                           :placeholder "Password"
                                           :required    true})]
                        [:div {:class "mt-6"}
                         (ui/button-primary {} "Sign in")])]))))

(defn authenticated-view
  [request]
  (let [user-id (get-in request [:session :user/id])]
    (list
     (ui/nav-header request "/")
     (ui/page-center {:class "space-y-4"}
                     (ui/page-title {:class "text-2xl"} "Welcome!")
                     (ui/text-muted {} (str "Signed in as user " user-id))
                     (form/form request :auth/sign-out
                       (ui/button-secondary {} "Sign out"))))))

;;; ----------------------------------------------------------------------------
;;; Actions

(defn- find-user-by-email
  "Look up user by email. Returns {:user/id :user/password-hash} or nil."
  [datahike email]
  (datahike/q datahike credential/user-by-email-query email))

(defn authenticate
  "Login action. Requires :keymaster, :datahike, and :postgres in request."
  [request]
  (span/with-span! {:name ::authenticate}
    (let [{:keys [datahike keymaster postgres]} (mw/request->state request)

          params                   (get-in request [:parameters :form])
          {:keys [email password]} params
          email-str                (cryptex/reveal email)
          ip-address               (request/remote-addr request)]
      (jdbc/with-transaction [tx (:datasource postgres)]
        (let [rate-check (rate-limit/check tx {:email      email-str
                                               :ip-address ip-address})]
          (if (anom/anomaly? rate-check)
            (do
              (log/info :msg        "Rate limited."
                        :email      email-str
                        :ip-address ip-address)
              (morph/respond (login-view request {:error (::anom/message rate-check)})))
            (let [user         (find-user-by-email datahike email-str)
                  has-user?    (some? user)
                  password-ok? (if has-user?
                                 (:valid (crypto/verify keymaster password (:user/password-hash user)))
                                 (do (crypto/verify keymaster password (:dummy-hash keymaster))
                                     false))]
              (rate-limit/record-attempt! tx {:email      email-str
                                              :ip-address ip-address
                                              :success    password-ok?})
              (if password-ok?
                (let [session-store (mw/request->session-store request)
                      old-sid       (get-in request [:session :sid])
                      new-sid       (session/rotate-session! session-store old-sid (:user/id user))]
                  (log/debug :msg     "Redirecting user..."
                             :user/id (:user/id user))
                  (morph/redirect "/" {:session (assoc (session/new-session session-store)
                                                       :sid     new-sid
                                                       :user/id (:user/id user))}))
                (morph/respond (login-view request {:error "Invalid email or password."}))))))))))

(defn sign-out
  [request]
  (span/with-span! {:name ::sign-out}
    (let [session-store (mw/request->session-store request)
          sid           (get-in request [:session :sid])]
      (when sid
        (session/clear-user! session-store sid))
      (morph/redirect "/" {:session (session/new-session session-store)}))))
