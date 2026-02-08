(ns bits.next
  (:require
   [bits.auth :as auth]
   [bits.html :as html]
   [bits.morph :as morph]
   [bits.ui :as ui]
   [clojure.string :as str]))

;;; ----------------------------------------------------------------------------
;;; Layout

(defn layout
  [_request & content]
  [:html {:class "min-h-screen" :lang "en"}
   [:head
    [:meta {:name "viewport" :content "width=device-width"}]
    [:title "Bits"]
    [:link {:rel "icon" :href "data:,"}]
    [:link {:rel "stylesheet" :href "/app.css"}]
    [:script {:src "/idiomorph@0.7.4.min.js"}]
    [:script {:src "/bits.js"}]]
   [:body {:class "min-h-screen bg-white dark:bg-neutral-950"}
    (into [:main#morph {:class "min-h-screen"}] content)]])

;;; ----------------------------------------------------------------------------
;;; Demo: Counter

(defonce !state (atom {:count 0}))

(defn email-form
  [{:keys [email error success]}]
  [:div#email-demo {:class "p-6 bg-white dark:bg-neutral-900 rounded-lg shadow max-w-sm"}
   [:h3 {:class "text-lg font-semibold mb-4 dark:text-white"} "Email Validation"]
   [:form {:class "space-y-4 transition-opacity inert:opacity-50 inert:cursor-wait"}
    (when error
      [:p {:class "text-sm text-red-600 dark:text-red-400"} error])
    (when success
      [:p {:class "text-sm text-green-600 dark:text-green-400"} success])
    [:input {:type        "email"
             :name        "email"
             :value       (or email "")
             :placeholder "you@example.com"
             :class       "w-full px-3 py-2 border rounded-md dark:bg-neutral-800 dark:border-neutral-700 dark:text-white"}]
    [:button {:type        "button"
              :data-action "email/validate"
              :class       "w-full px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"}
     "Submit"]]])

(defn redirect-demo
  []
  [:div {:class "p-6 bg-white dark:bg-neutral-900 rounded-lg shadow max-w-sm"}
   [:h3 {:class "text-lg font-semibold mb-4 dark:text-white"} "Redirect Demo"]
   [:button {:type        "button"
             :data-action "demo/redirect"
             :class       "px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-500"}
    "Go to example.com"]])

(defn counter-view
  [_request]
  (list
   (ui/nav-header "/")
   [:div {:class "min-h-screen flex flex-col justify-center items-center space-y-2"}
    [:header
     [:h1 {:class "text-4xl dark:text-neutral-100"}
      "Count: "
      [:span {:class "font-bold"} (:count @!state)]]]
    [:section {:class "flex space-x-2"}
     [:button
      {:type        "button"
       :class       "rounded-full bg-indigo-600 p-2 text-white shadow-xs hover:bg-indigo-500 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 dark:bg-indigo-500 dark:shadow-none dark:hover:bg-indigo-400 dark:focus-visible:outline-indigo-500"
       :data-action "counter/inc"}
      [:svg
       {:viewBox     "0 0 20 20"
        :fill        "currentColor"
        :data-slot   "icon"
        :aria-hidden "true"
        :class       "size-5"}
       [:path
        {:d "M10.75 4.75a.75.75 0 0 0-1.5 0v4.5h-4.5a.75.75 0 0 0 0 1.5h4.5v4.5a.75.75 0 0 0 1.5 0v-4.5h4.5a.75.75 0 0 0 0-1.5h-4.5v-4.5Z"}]]]
     [:button
      {:type        "button"
       :class       "rounded-full bg-indigo-600 p-2 text-white shadow-xs hover:bg-indigo-500 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 dark:bg-indigo-500 dark:shadow-none dark:hover:bg-indigo-400 dark:focus-visible:outline-indigo-500"
       :data-action "counter/dec"}
      [:svg
       {:viewBox     "0 0 20 20"
        :fill        "currentColor"
        :data-slot   "icon"
        :aria-hidden "true"
        :class       "size-5"}
       [:path
        {:d "M4.75 9.25a.75.75 0 0 0 0 1.5h10.5a.75.75 0 0 0 0-1.5H4.75Z"}]]]]]))

(defn email-view
  ([_request] (email-view _request {}))
  ([_request form-state]
   (list
    (ui/nav-header "/email")
    [:div {:class "min-h-screen flex flex-col justify-center items-center"}
     (email-form form-state)])))

(defn redirect-view
  [_request]
  (list
   (ui/nav-header "/redirect")
   [:div {:class "min-h-screen flex flex-col justify-center items-center"}
    (redirect-demo)]))

;;; ----------------------------------------------------------------------------
;;; Demo: Cursors

(defonce !cursors (atom {}))

(defn update-cursor!
  [channel-id x y]
  (swap! !cursors assoc channel-id [x y (System/currentTimeMillis)]))

(defn remove-cursor!
  [channel-id]
  (swap! !cursors dissoc channel-id))

(def cursor-colors
  ["bg-red-500"    "bg-blue-500"   "bg-green-500"  "bg-yellow-500"
   "bg-purple-500" "bg-pink-500"   "bg-indigo-500" "bg-teal-500"
   "bg-orange-500" "bg-cyan-500"   "bg-lime-500"   "bg-rose-500"])

(defn cursor-color
  [channel-id]
  (nth cursor-colors (mod (hash channel-id) (count cursor-colors))))

(defn cursor-styles
  [cursors]
  [:style {:id "cursor-positions"}
   (html/raw
    (str/join "\n"
              (for [[channel-id [x y _]] cursors]
                (format ".cursor[data-channel=\"%s\"] { left: %dpx; top: %dpx; }"
                        (subs channel-id 0 6) x y))))])

(defn cursor-label
  [channel-id]
  (let [short-id (subs channel-id 0 6)
        color    (cursor-color channel-id)]
    [:div {:class "cursor" :data-channel short-id}
     [:span {:class (str "px-1.5 py-0.5 text-xs font-mono rounded text-white " color)}
      short-id]]))

(defn cursors-view
  [_request]
  (let [cursors @!cursors]
    (list
     (ui/nav-header "/cursors")
     [:div {:id               "cursor-container"
            :class            "relative min-h-screen"
            :data-track-mouse "cursor/move"}

      (cursor-styles cursors)

      (for [[cid _] cursors]
        (cursor-label cid))

      [:div {:class "flex flex-col justify-center items-center min-h-screen"}
       [:h1 {:class "text-4xl font-bold dark:text-white"} "Presence Cursors"]
       [:p {:class "text-neutral-500 mt-4"}
        (str (count cursors) " cursor" (when (not= 1 (count cursors)) "s") " active")]]])))

;;; ----------------------------------------------------------------------------
;;; Actions
;;;
;;; Raw action definitions. Plain functions need no params. Use a map with
;;; :handler and :params when the action requires coerced parameters.
;;; Normalized at system start, not at load time.

(def actions
  {:auth/login     {:handler auth/authenticate
                    :params  [[:email :email]
                              [:password :password]]}
   :auth/sign-out  auth/sign-out
   :counter/inc    (fn [_req] (swap! !state update :count inc))
   :counter/dec    (fn [_req] (swap! !state update :count dec))
   :demo/redirect  (fn [_req] (morph/redirect "https://example.com"))
   :email/validate (fn [request]
                     (let [email (get-in request [:params "email"] "")]
                       (cond
                         (str/blank? email)
                         (morph/respond (email-view request {:email email
                                                             :error "Email is required"}))

                         (not (str/includes? email "@"))
                         (morph/respond (email-view request {:email email
                                                             :error "Please enter a valid email address"}))

                         :else
                         (morph/respond (email-view request {:email   email
                                                             :success (str "Welcome, " email "!")})))))
   :cursor/move    (fn [request]
                     (let [channel-id (get-in request [:params "channel"])
                           x          (parse-long (get-in request [:params "x"] "0"))
                           y          (parse-long (get-in request [:params "y"] "0"))]
                       (when (and channel-id x y (< x 10000) (< y 10000))
                         (update-cursor! channel-id x y))))})

;;; ----------------------------------------------------------------------------
;;; Routes

(defn- login-view-wrapper
  [request]
  (auth/login-view request {}))

(def routes
  [["/"
    {:get  (morph/page-handler layout counter-view)
     :post (morph/render-handler counter-view)}]
   ["/cursors"
    {:get  (morph/page-handler layout cursors-view)
     :post (morph/render-handler cursors-view {:on-close remove-cursor!})}]
   ["/email"
    {:get  (morph/page-handler layout email-view)
     :post (morph/render-handler email-view)}]
   ["/login"
    {:get  (morph/page-handler layout login-view-wrapper)
     :post (morph/render-handler login-view-wrapper)}]
   ["/redirect"
    {:get  (morph/page-handler layout redirect-view)
     :post (morph/render-handler redirect-view)}]])
