(ns bits.auth
  (:require
   [bits.auth.credential :as credential]
   [bits.auth.rate-limit :as rate-limit]
   [bits.crypto :as crypto]
   [bits.cryptex :as cryptex]
   [bits.html :as html]
   [bits.next.session :as session]
   [io.pedestal.log :as log]
   [steffan-westcott.clj-otel.api.trace.span :as span]))

;;; ----------------------------------------------------------------------------
;;; OWASP: Generic error message — never reveal whether email exists
;;;
;;; "The application should respond with a generic error message regardless
;;; of whether the user ID or password was incorrect. It should also return
;;; the same HTTP status code." — OWASP Authentication Cheat Sheet

(def ^:private generic-error-message
  "Invalid email address or password.")

;;; ----------------------------------------------------------------------------
;;; Views

(defn login-view
  "Login form. Shown as a full page and also returned inline on validation
   errors via the respond pattern (PR #2)."
  [request & {:keys [error]}]
  (let [csrf (::bits.next/csrf request)]
    (list
     [:div {:class "min-h-screen flex flex-col justify-center items-center"}
      [:div {:class "w-full max-w-sm space-y-8"}
       [:div
        [:h1 {:class "text-center text-2xl font-bold tracking-tight text-neutral-900 dark:text-neutral-100"}
         "Sign in to your account"]]
       [:form#login-form {:method "post" :action "/action"}
        [:input {:type "hidden" :name "action" :value "authenticate"}]
        [:input {:type "hidden" :name "csrf" :value csrf}]
        (when error
          [:div {:class "rounded-md bg-red-50 p-4 mb-4 dark:bg-red-500/15 dark:outline dark:outline-red-500/25"
                 :role  "alert"}
           [:div {:class "flex"}
            [:div {:class "shrink-0"}
             [:svg {:viewBox "0 0 20 20" :fill "currentColor" :class "size-5 text-red-400" :aria-hidden "true"}
              [:path {:d         "M10 18a8 8 0 1 0 0-16 8 8 0 0 0 0 16ZM8.28 7.22a.75.75 0 0 0-1.06 1.06L8.94 10l-1.72 1.72a.75.75 0 1 0 1.06 1.06L10 11.06l1.72 1.72a.75.75 0 1 0 1.06-1.06L11.06 10l1.72-1.72a.75.75 0 0 0-1.06-1.06L10 8.94 8.28 7.22Z"
                     :clip-rule "evenodd"
                     :fill-rule "evenodd"}]]]
            [:div {:class "ml-3"}
             [:p {:class "text-sm font-medium text-red-800 dark:text-red-200"} error]]]])
        [:div {:class "space-y-0"}
         [:div
          [:input#email
           {:name         "email"
            :type         "email"
            :autocomplete "email"
            :required     true
            :aria-label   "Email address"
            :class        "block w-full rounded-t-md bg-white px-3 py-1.5 text-base text-neutral-900 dark:text-neutral-100 dark:bg-neutral-800 outline-1 -outline-offset-1 outline-neutral-300 dark:outline-neutral-600 placeholder:text-neutral-400 focus:relative focus:outline-2 focus:-outline-offset-2 focus:outline-indigo-600 sm:text-sm/6"
            :placeholder  "Email address"}]]
         [:div {:class "-mt-px"}
          [:input#password
           {:name         "password"
            :type         "password"
            :autocomplete "current-password"
            :required     true
            :aria-label   "Password"
            :class        "block w-full rounded-b-md bg-white px-3 py-1.5 text-base text-neutral-900 dark:text-neutral-100 dark:bg-neutral-800 outline-1 -outline-offset-1 outline-neutral-300 dark:outline-neutral-600 placeholder:text-neutral-400 focus:relative focus:outline-2 focus:-outline-offset-2 focus:outline-indigo-600 sm:text-sm/6"
            :placeholder  "Password"}]]]
        [:div {:class "mt-6"}
         [:button
          {:type  "submit"
           :class "flex w-full justify-center rounded-md bg-indigo-600 px-3 py-1.5 text-sm/6 font-semibold text-white hover:bg-indigo-500 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"}
          "Sign in"]]]]])))

(defn authenticated-view
  "Simple view shown when the user is authenticated."
  [request]
  (let [csrf    (::bits.next/csrf request)
        user-id (get-in request [:session :user-id])]
    (list
     [:div {:class "min-h-screen flex flex-col justify-center items-center space-y-4"}
      [:h1 {:class "text-2xl font-bold text-neutral-900 dark:text-neutral-100"}
       "Welcome"]
      [:p {:class "text-neutral-500"}
       (str "Signed in as " user-id)]
      [:form {:method "post" :action "/action"}
       [:input {:type "hidden" :name "action" :value "sign-out"}]
       [:input {:type "hidden" :name "csrf" :value csrf}]
       [:button
        {:type  "submit"
         :class "rounded-md bg-neutral-600 px-3 py-1.5 text-sm/6 font-semibold text-white hover:bg-neutral-500"}
        "Sign out"]]])))

;;; ----------------------------------------------------------------------------
;;; Authentication logic

(def ^:private dummy-hash
  "Pre-computed Argon2id hash of a dummy password. Used to ensure the
   verify code path runs even when the email doesn't exist, preventing
   timing oracles on email existence."
  (delay (crypto/derive (cryptex/cryptex "constant-time-dummy-password-bits"))))

(defn- constant-time-verify
  "Verify a password against a dummy hash to prevent timing oracle.
   Both the user-found and user-not-found paths call crypto/verify,
   ensuring identical timing."
  [password]
  (crypto/verify (cryptex/cryptex (or password "")) @dummy-hash))

(defn authenticate
  "The authenticate action. Looks up credentials in Datahike, verifies
   the password with Argon2id, rotates the session on success.

   On failure, returns the login form with a generic error via respond.
   On success, returns a redirect event via respond.

   Never reveals whether the email exists."
  [request]
  (span/with-span! {:name ::authenticate}
    (let [database   (::database request)
          pool       (::pool request)
          email      (get-in request [:params "email"])
          password   (get-in request [:params "password"])
          ip-address (:remote-addr request)
          sid        (get-in request [:session :sid])]

      ;; Check rate limit first
      (if-let [throttle (rate-limit/throttled? pool {:email      (or email "")
                                                     :ip-address ip-address})]
        (do
          (log/warn :msg "Authentication throttled" :reason (:reason throttle) :email email)
          {::respond (login-view request :error "Too many attempts. Please try again later.")})

        ;; Proceed with authentication
        (let [user (when (and email (seq email))
                     (credential/find-by-email database email))

              {:keys [valid]}
              (if user
                (crypto/verify (cryptex/cryptex (or password ""))
                               (:user/password-hash user))
                ;; No user found — still verify to prevent timing oracle
                (do (constant-time-verify password)
                    {:valid false}))]

          ;; Record the attempt (success or failure) for rate limiting
          (rate-limit/record-attempt! pool {:email      (or email "")
                                            :ip-address ip-address
                                            :success    (boolean valid)})

          (if valid
            ;; Success: rotate session (prevents session fixation), redirect.
            ;; wrap-csrf detects the new SID and updates the CSRF cookie.
            (let [idle-days 30
                  new-sid   (session/rotate-session! pool sid (:user/id user) idle-days)]
              (log/info :msg "Authentication successful" :user-id (:user/id user))
              {::respond [:script (html/raw "window.location.href = '/';")]
               ::session {:sid     new-sid
                          :user-id (:user/id user)}})

            ;; Failure: generic error, no hint about email existence
            (do
              (log/info :msg "Authentication failed" :email email)
              {::respond (login-view request :error generic-error-message)})))))))

(defn sign-out
  "Clear session user, redirect to login."
  [request]
  (span/with-span! {:name ::sign-out}
    (let [pool (::pool request)
          sid  (get-in request [:session :sid])]
      (when sid
        (session/clear-user! pool sid 30))
      {::respond [:script (html/raw "window.location.href = '/login';")]})))
