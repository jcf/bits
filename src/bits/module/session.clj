(ns bits.module.session
  (:require
   [bits.anomaly :as anom]
   [bits.auth.credential :as credential]
   [bits.auth.rate-limit :as rate-limit]
   [bits.cryptex :as cryptex]
   [bits.crypto :as crypto]
   [bits.datomic :as datomic]
   [bits.form :as form]
   [bits.locale :refer [tru]]
   [bits.middleware :as mw]
   [bits.morph :as morph]
   [bits.postgres :as postgres]
   [bits.request :as request]
   [bits.session :as session]
   [bits.ui :as ui]
   [datomic.api :as d]
   [io.pedestal.log :as log]
   [next.jdbc :as jdbc]
   [steffan-westcott.clj-otel.api.trace.span :as span]))

;;; ----------------------------------------------------------------------------
;;; Views

(defn login-view
  [request opts]
  (let [{:keys [auth-failed? action-error]} opts
        action-failed?                      (or auth-failed? action-error)
        {:keys [ring bg shadow]}            (get form/form-classes
                                                 (if action-failed? :bits.form/error :bits.form/pristine)
                                                 (:bits.form/pristine form/form-classes))
        button-text                         (cond
                                              action-error action-error
                                              auth-failed? (tru "Invalid credentials")
                                              :else        (tru "Sign in"))]
    (list
     (ui/nav-header request "/login")
     (ui/page-center {:class ["px-6" "py-12" "lg:px-8"]}
       [:div {:class ["sm:mx-auto" "sm:w-full" "sm:max-w-sm"]}
        [:h2 {:class ["mt-10" "text-center" "text-2xl/9" "font-bold"
                      "tracking-tight" "text-primary"]}
         (tru "Sign in to your account")]]
       [:div {:class ["mt-10" "sm:mx-auto" "sm:w-full" "sm:max-w-sm"]}
        (form/form request :auth/login
          (cond->
           {:class (str "rounded-xl p-6 transition-all duration-500 ease-out "
                        ring " " shadow " " bg " "
                        (when action-failed? "form-shake"))}
            action-failed? (assoc :data-reset true))
          [:div {:class "space-y-1"}
           (form/validated-field
            {:name         :email
             :label        (tru "Email")
             :type         "email"
             :placeholder  "creator@bits.page"
             :autocomplete "username"})
           (form/validated-field
            {:name         :password
             :label        (tru "Password")
             :type         "password"
             :placeholder  "correct-horse-battery-staple"
             :autocomplete "current-password"})]
          [:div {:class "mt-4"}
           (let [base-classes   "w-full py-2.5 px-4 rounded-lg text-sm font-medium transition-all duration-300 ease-out cursor-pointer"
                 error-classes  "bg-red-500/20 text-red-400 ring-2 ring-red-500/50"
                 normal-classes "bg-white/[0.08] text-zinc-300 ring-1 ring-white/10 hover:bg-white/[0.12] hover:ring-white/20"]
             [:button {:type  "submit"
                       :name  "submit"
                       :value "true"
                       :class (str base-classes " " (if action-failed? error-classes normal-classes))}
              button-text])])]))))

(defn authenticated-view
  [request]
  (let [user-id (get-in request [:session :user/id])]
    (list
     (ui/nav-header request "/")
     (ui/page-center {:class "space-y-4"}
       (ui/page-title {:class "text-2xl"} (tru "Welcome!"))
       (ui/text-muted {} (tru "Signed in as user {0}" user-id))
       (form/form request :auth/sign-out
         (ui/button-secondary {} (tru "Sign out")))))))

;;; ----------------------------------------------------------------------------
;;; Actions

(defn- find-user-by-email
  [database email]
  (d/q credential/user-by-email-query (datomic/db database) email))

(defn authenticate
  [request]
  (span/with-span! {:name ::authenticate}
    (when (::form/submitted? (::form/form request))
      (let [params                                            (get-in request [:parameters :form])
            {:keys [datomic keymaster postgres rate-limiter]} (mw/request->state request)
            tenant-id                                         (get-in request [:session/realm :tenant/id])
            email                                             (:email params)
            password                                          (:password params)
            email-str                                         (cryptex/reveal email)
            ip-address                                        (request/remote-addr request)]
        (jdbc/with-transaction [tx (:datasource postgres)]
          (let [limiter    (assoc rate-limiter :postgres (postgres/assoc-conn postgres tx))
                rate-check (rate-limit/check limiter tenant-id {:email      email-str
                                                                :ip-address ip-address})]
            (if (anom/anomaly? rate-check)
              (do
                (log/info :msg        "Rate limited."
                          :email      email-str
                          :ip-address ip-address)
                (morph/respond (login-view request {:action-error (::anom/message rate-check)})))
              (let [user         (find-user-by-email datomic email-str)
                    has-user?    (some? user)
                    password-ok? (if has-user?
                                   (:valid (crypto/verify keymaster password (:user/password-hash user)))
                                   (do (crypto/verify keymaster password (:dummy-hash keymaster))
                                       false))]
                (rate-limit/record-attempt! limiter tenant-id {:email      email-str
                                                               :ip-address ip-address
                                                               :success    password-ok?})
                (if password-ok?
                  (let [session-store (mw/request->session-store request)
                        old-sid       (get-in request [:session :sid])
                        new-sid       (session/rotate-session! session-store tenant-id old-sid (:user/id user))]
                    (log/debug :msg     "Redirecting user..."
                               :user/id (:user/id user))
                    (morph/redirect "/" {:session (assoc (session/new-session session-store)
                                                         :sid     new-sid
                                                         :user/id (:user/id user))}))
                  (morph/respond (login-view request {:auth-failed? true})))))))))))

(defn sign-out
  [request]
  (span/with-span! {:name ::sign-out}
    (let [session-store (mw/request->session-store request)
          tenant-id     (get-in request [:session/realm :tenant/id])
          sid           (get-in request [:session :sid])]
      (when sid
        (session/clear-user! session-store tenant-id sid))
      (morph/redirect "/" {:session (session/new-session session-store)}))))

;;; ----------------------------------------------------------------------------
;;; Module

(def module
  {:name    :bits.module/session
   :routes  [["/login" (assoc (morph/morphable ui/layout #(login-view % {}))
                              :bits/page (fn [_request] {:page/title (tru "Login")}))]]
   :actions {:auth/login    {:handler authenticate
                             :params  [[:email :email]
                                       [:password :password]]}
             :auth/sign-out sign-out}})
