(ns bits.next
  (:require
   [bits.anomaly :as anom]
   [bits.auth :as auth]
   [bits.html :as html]
   [bits.middleware :as mw]
   [bits.morph :as morph]
   [bits.ui :as ui]
   [clojure.string :as str]))

;;; ----------------------------------------------------------------------------
;;; Demo: Counter

(defonce !state (atom {:count 0}))

(defn email-form
  [{:keys [email error success]}]
  (ui/card {:id "email-demo"}
           (ui/card-title "Email Validation")
           [:form {:class "space-y-4 transition-opacity inert:opacity-50 inert:cursor-wait"}
            (when error (ui/text-error error))
            (when success (ui/text-success success))
            (ui/input {:type        "email"
                       :name        "email"
                       :value       (or email "")
                       :class       "rounded-md"
                       :placeholder "you@example.com"})
            (ui/button-primary {:type        "button"
                                :data-action "email/validate"}
                               "Submit")]))

(defn redirect-demo
  []
  (ui/card {}
           (ui/card-title "Redirect Demo")
           (ui/button-primary {:type        "button"
                               :data-action "demo/redirect"}
                              "Go to example.com")))

(def ^:private plus-icon
  [:svg {:viewBox "0 0 20 20" :fill "currentColor" :aria-hidden "true" :class "size-5"}
   [:path {:d "M10.75 4.75a.75.75 0 0 0-1.5 0v4.5h-4.5a.75.75 0 0 0 0 1.5h4.5v4.5a.75.75 0 0 0 1.5 0v-4.5h4.5a.75.75 0 0 0 0-1.5h-4.5v-4.5Z"}]])

(def ^:private minus-icon
  [:svg {:viewBox "0 0 20 20" :fill "currentColor" :aria-hidden "true" :class "size-5"}
   [:path {:d "M4.75 9.25a.75.75 0 0 0 0 1.5h10.5a.75.75 0 0 0 0-1.5H4.75Z"}]])

(defn counter-view
  [request]
  (list
   (ui/nav-header request "/")
   (ui/page-center {:class "space-y-2"}
                   [:header
                    (ui/page-title {}
                                   "Count: "
                                   [:span {:class "font-bold"} (:count @!state)])]
                   [:section {:class "flex space-x-2"}
                    (ui/icon-button {:data-action "counter/inc"} plus-icon)
                    (ui/icon-button {:data-action "counter/dec"} minus-icon)])))

(defn email-view
  ([request] (email-view request {}))
  ([request form-state]
   (list
    (ui/nav-header request "/email")
    (ui/page-center {}
                    (email-form form-state)))))

(defn redirect-view
  [request]
  (list
   (ui/nav-header request "/redirect")
   (ui/page-center {}
                   (redirect-demo))))

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
  [request cursors]
  [:style {:id "cursor-positions" :nonce (mw/request->nonce request)}
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
  [request]
  (let [cursors @!cursors]
    (list
     (ui/nav-header request "/cursors")
     [:div {:id               "cursor-container"
            :class            "relative min-h-screen"
            :data-track-mouse "cursor/move"}

      (cursor-styles request cursors)

      (for [[cid _] cursors]
        (cursor-label cid))

      (ui/page-center {}
                      (ui/page-title {} "Presence Cursors")
                      (ui/text-muted {:class "mt-4"}
                                     (str (count cursors) " cursor" (when (not= 1 (count cursors)) "s") " active")))])))

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
;;; Realm

(defn realm-not-found-view
  [_request]
  (ui/page-center
   {}
   (ui/page-title {} "Realm not found")
   (ui/text-muted {:class "mt-4"}
                  "Want your own Bits? We want to hear from you!")))

;;; ----------------------------------------------------------------------------
;;; Routes

(defn- login-view-wrapper
  [request]
  (auth/login-view request {}))

(defn morphable
  ([layout-fn view-fn]
   (morphable layout-fn view-fn {}))
  ([layout-fn view-fn options]
   (let [wrapped (fn [request]
                   (cond
                     (anom/anomaly? (:session/realm request))
                     (realm-not-found-view request)
                     :else
                     (view-fn request)))]
     {:get  (fn realm-aware-page-handler
              [request]
              (let [respond (morph/page-handler layout-fn wrapped)]
                (if (anom/anomaly? (:session/realm request))
                  {:status  404
                   :headers {"content-type" "text/html; charset=utf-8"}
                   :body    (html/html (layout-fn request (realm-not-found-view request)))}
                  (respond request))))
      :post (morph/render-handler wrapped options)})))

(def routes
  [["/"         (morphable ui/layout counter-view)]
   ["/cursors"  (morphable ui/layout cursors-view {:on-close remove-cursor!})]
   ["/email"    (morphable ui/layout email-view)]
   ["/login"    (morphable ui/layout login-view-wrapper)]
   ["/redirect" (morphable ui/layout redirect-view)]])
