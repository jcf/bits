(ns bits.service.session
  (:require
   [bits.assets :as assets]
   [bits.cryptex :as cryptex]
   [bits.crypto :as crypto]
   [bits.csrf :as csrf]
   [bits.html :as html]
   [bits.interceptor :as i]
   [bits.tailwind :as tw]
   [clojure.core.async :as async]
   [io.pedestal.http.body-params :as body-params]
   [io.pedestal.http.route :as route]
   [io.pedestal.interceptor :refer [interceptor]]
   [io.pedestal.log :as log]
   [java-time.api :as time]
   [ring.util.response :as response]))

(defn- sign-in-form
  [request]
  (let [verifier     (csrf/anti-forgery-token request)
        session-path (route/url-for :bits.route/post-sign-in)]
    [:form
     {:class   "space-y-6"
      :action  session-path
      :method  "post"
      :hx-post session-path}
     (tw/alert-with-list {:title "Invalid credentials"
                          :list ["Please verify your email address and password."]})
     [:input {:type "hidden" :name "verifier" :value verifier}]
     [:div
      [:div
       {:class "col-span-2"}
       [:input#email-address
        {:name         "email"
         :type         "email"
         :autocomplete "email"
         :aria-label   "Email address"
         :class
         "block w-full rounded-t-md bg-white px-3 py-1.5 text-base text-neutral-900 dark:text-neutral-100 outline-1 -outline-offset-1 outline-neutral-300 placeholder:text-neutral-400 focus:relative focus:outline-2 focus:-outline-offset-2 focus:outline-blue-600 sm:text-sm/6"
         :placeholder  "Email address"}]]
      [:div
       {:class "-mt-px"}
       [:input#password
        {:name         "password"
         :type         "password"
         :autocomplete "current-password"
         :aria-label   "Password"
         :class
         "block w-full rounded-b-md bg-white px-3 py-1.5 text-base text-neutral-900 dark:text-neutral-100 outline-1 -outline-offset-1 outline-neutral-300 placeholder:text-neutral-400 focus:relative focus:outline-2 focus:-outline-offset-2 focus:outline-blue-600 sm:text-sm/6"
         :placeholder  "Password"}]]]
     [:div
      {:class "flex items-center justify-between"}
      [:div
       {:class "flex gap-3"}
       [:div
        {:class "flex h-6 shrink-0 items-center"}
        [:div
         {:class "group grid size-4 grid-cols-1"}
         [:input#remember-me
          {:name "remember-me"
           :type "checkbox"
           :class
           "col-start-1 row-start-1 appearance-none rounded-sm border border-neutral-300 bg-white checked:border-blue-600 checked:bg-blue-600 indeterminate:border-blue-600 indeterminate:bg-blue-600 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-600 disabled:border-neutral-300 disabled:bg-neutral-100 disabled:checked:bg-neutral-100 forced-colors:appearance-auto"}]
         [:svg
          {:class
           "pointer-events-none col-start-1 row-start-1 size-3.5 self-center justify-self-center stroke-white group-has-disabled:stroke-neutral-950/25"
           :viewBox "0 0 14 14"
           :fill    "none"}
          [:path
           {:class           "opacity-0 group-has-checked:opacity-100"
            :d               "M3 8L6 11L11 3.5"
            :stroke-width    "2"
            :stroke-linecap  "round"
            :stroke-linejoin "round"}]
          [:path
           {:class           "opacity-0 group-has-indeterminate:opacity-100"
            :d               "M3 7H11"
            :stroke-width    "2"
            :stroke-linecap  "round"
            :stroke-linejoin "round"}]]]]
       [:label
        {:for "remember-me" :class "block text-sm/6 text-neutral-900 dark:text-neutral-100"}
        "Remember me"]]
      [:div
       {:class "text-sm/6"}
       [:a
        {:href  "#"
         :class "font-semibold text-blue-600 hover:text-blue-500"}
        "Forgot password?"]]]
     [:div
      [:button
       {:type "submit"
        :class
        "flex w-full justify-center rounded-md bg-blue-600 px-3 py-1.5 text-sm/6 font-semibold text-white hover:bg-blue-500 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-600"}
       "Sign in"]]]))

(defn get-sign-in
  [request]
  {:status  200
   :headers {"Content-Type" "text/html; charset=utf-8"}
   :body
   (html/html
    (html/layout
     request
     [:div
      {:class
       "flex min-h-full items-center justify-center px-4 py-12 sm:px-6 lg:px-8"}
      [:div
       {:class "w-full max-w-sm space-y-10"}
       [:div
        [:span
         {:class "mx-auto h-10 w-auto text-neutral-100 dark:text-neutral-900"} "Bits"]
        [:h2
         {:class
          "mt-10 text-center text-2xl/9 font-bold tracking-tight text-neutral-900 dark:text-neutral-100"}
         "Sign in to your account"]]
       (sign-in-form request)]]))})

(def verify-credentials
  (interceptor
   {:name ::verify-credentials
    :enter
    (fn enter-verify-credentials
      [context]
      (async/thread
        (try
          (let [{:keys [email password]} (get-in context [:request :form-params])
                ;; Ensure we won't accidentally leak the password downstream.
                _                        (assert (or (nil? password) (cryptex/cryptex? password)))

                ;; TODO Find credentials
                credentials nil

                {:keys [valid update]} (when (and (some? password) (some? credentials))
                                         (crypto/verify password (:credential/password-hash credentials)))]
            (log/info :in                ::verify-credentials
                      :credentials/email email
                      :valid             valid
                      :update            update)
            (cond-> context
              (true? valid)
              (assoc-in [:request :bits/operator] {:credential/email email})))
          (catch Exception ex
            (assoc context :io.pedestal.interceptor.chain/error ex)))))}))

(defn post-sign-in
  [request]
  (let [operator (:bits/operator request)]
    (if (some? operator)
      (assoc (response/redirect (route/url-for :bits.route/get-root))
             :session (random-uuid))
      {:status  403
       :headers {"Content-Type" "text/plain"}
       :body    "What's the magic word?\n"})))

(defn hx
  [page-fn hx-fn]
  (interceptor
   {:name ::hx
    :enter
    (fn enter-hx
      [{:keys [request] :as context}]
      (let [respond (if (some? (response/get-header request "hx-request"))
                      page-fn
                      hx-fn)]
        (assoc context :response (respond request))))}))

(def routes
  #{["/login"
     :get [i/error-interceptor
           `get-sign-in]
     :route-name :bits.route/get-sign-in]
    ["/session"
     :post [i/error-interceptor
            (i/protect-params-interceptor #{[:form-params :password]})
            verify-credentials
            (hx post-sign-in sign-in-form)]
     :route-name :bits.route/post-sign-in]})
