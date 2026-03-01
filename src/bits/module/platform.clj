(ns bits.module.platform
  (:require
   [bits.form :as form]
   [bits.html :as html]
   [bits.locale :refer [tru]]
   [bits.middleware :as mw]
   [bits.morph :as morph]
   [bits.ui :as ui]
   [clojure.string :as str]
   [datomic.api :as d]))

;;; ----------------------------------------------------------------------------
;;; Counter

(defonce !counter (atom {:count 0}))

(def ^:private plus-icon
  [:svg {:viewBox "0 0 20 20" :fill "currentColor" :aria-hidden "true" :class "size-5"}
   [:path {:d "M10.75 4.75a.75.75 0 0 0-1.5 0v4.5h-4.5a.75.75 0 0 0 0 1.5h4.5v4.5a.75.75 0 0 0 1.5 0v-4.5h4.5a.75.75 0 0 0 0-1.5h-4.5v-4.5Z"}]])

(def ^:private minus-icon
  [:svg {:viewBox "0 0 20 20" :fill "currentColor" :aria-hidden "true" :class "size-5"}
   [:path {:d "M4.75 9.25a.75.75 0 0 0 0 1.5h10.5a.75.75 0 0 0 0-1.5H4.75Z"}]])

(defn counter-view
  [request]
  (list
   (ui/nav-header request "/counter")
   (ui/page-center {:class "space-y-2"}
     [:header
      (ui/page-title {}
        (tru "Count: ")
        [:span {:class "font-bold"} (:count @!counter)])]
     [:section {:class "flex space-x-2"}
      (ui/icon-button {:data-action "counter/dec" :aria-label (tru "Decrement")} minus-icon)
      (ui/icon-button {:data-action "counter/inc" :aria-label (tru "Increment")} plus-icon)])))

;;; ----------------------------------------------------------------------------
;;; Form Demo

(def Username
  [:and
   [:string {:min 3 :error/message "At least 3 characters"}]
   [:re {:error/message "Letters, numbers, underscores only"}
    #"^[a-zA-Z0-9_]+$"]])

(def Email
  [:and
   [:string {:min 1}]
   [:re {:error/message "Enter a valid email address"}
    #"^[^\s@]+@[^\s@]+\.[^\s@]+$"]])

(def Country
  [:enum {:error/message "Select a country"} "us" "uk" "ca" "au" "de" "fr" "jp"])

(def form-schema
  {:username Username
   :email    Email
   :country  Country})

(defn- form-demo
  [request {:keys [validation success?]}]
  (let [validation               (when-not success? validation)
        form-status              (form/form-status validation)
        {:keys [ring bg shadow]} (get form/form-classes form-status)
        error?                   (= form-status :bits.form/error)]
    (form/form request :demo/validate
      (cond-> {:class (str "rounded-xl p-6 transition-all duration-500 ease-out "
                           ring " " shadow " " bg " "
                           (when error? "form-shake"))}
        success? (assoc :data-reset true))
      [:div {:class "space-y-2"}
       (form/validated-field
        {:name           :username
         :label          (tru "Username")
         :placeholder    "jcf_rocks"
         :autocomplete   "off"
         :data-1p-ignore true}
        (get validation :username))
       (form/validated-field
        {:name           :email
         :label          (tru "Email")
         :type           "email"
         :placeholder    "you@example.com"
         :autocomplete   "off"
         :data-1p-ignore true}
        (get validation :email))
       [:div
        [:label {:class "block text-xs font-medium tracking-wide text-zinc-500 uppercase mb-1.5 pl-0.5"}
         (tru "Country")]
        [:select {:name         "country"
                  :autocomplete "country-name"
                  :class        "w-full px-3.5 py-2.5 rounded-lg text-sm bg-white/[0.04] ring-1 ring-white/10 text-zinc-200 outline-1 outline-offset-1 outline-transparent focus-visible:outline-accent transition-all duration-300 ease-out"}
         [:option {:value ""} (tru "Select a country")]
         [:option {:value "us"} "United States"]
         [:option {:value "uk"} "United Kingdom"]
         [:option {:value "ca"} "Canada"]
         [:option {:value "au"} "Australia"]
         [:option {:value "de"} "Germany"]
         [:option {:value "fr"} "France"]
         [:option {:value "jp"} "Japan"]]
        [:div {:class "h-5"}]]]
      [:div {:class "mt-4"}
       (let [base-classes    "block w-full py-3.5 border-none rounded-lg font-sans text-[0.9375rem] font-semibold cursor-pointer tracking-wide transition-opacity duration-150"
             error-classes   "bg-red-500/80 text-surface hover:opacity-90"
             success-classes "bg-accent text-surface hover:opacity-90"
             normal-classes  "bg-white/[0.08] text-zinc-300 hover:opacity-80"]
         [:button {:type  "submit"
                   :name  "submit"
                   :value "true"
                   :class (str base-classes " " (cond error?   error-classes
                                                      success? success-classes
                                                      :else    normal-classes))}
          (cond error?   (tru "Whoops!")
                success? (tru "Success!")
                :else    (tru "Submit"))])])))

(defn form-view
  ([request] (form-view request {}))
  ([request opts]
   (list
    (ui/nav-header request "/form")
    (ui/page-center {:class ["px-6" "py-12" "lg:px-8"]}
      [:div {:class ["sm:mx-auto" "sm:w-full" "sm:max-w-sm"]}
       [:h2 {:class ["mt-10" "text-center" "text-2xl/9" "font-bold"
                     "tracking-tight" "text-primary"]}
        (tru "Forms")]]
      [:div {:class ["mt-10" "sm:mx-auto" "sm:w-full" "sm:max-w-sm"]}
       (form-demo request opts)]))))

;;; ----------------------------------------------------------------------------
;;; Explore

(defn explore-view
  [request]
  (let [db      (mw/request->db request)
        tenants (sort-by (some-fn :creator/display-name :creator/handle)
                         (d/q '[:find [(pull ?e [:creator/display-name
                                                 :creator/handle
                                                 {:tenant/domains [:domain/name]}]) ...]
                                :where [?e :creator/handle]]
                              db))]
    (list
     (ui/nav-header request "/")
     (ui/page-center
      {}
       (if-not (seq tenants)
         (list
          (ui/page-title {} (tru "No tenants found"))
          (ui/text-muted {:class ["mt-4"]}
            (tru "Please create a tenant or two.")))
         [:ul {:class "space-y-2"}
          (for [{:keys [creator/display-name
                        tenant/domains
                        creator/handle]} (sort-by :creator/handle tenants)
                :let                     [{domain-name :domain/name} (first domains)]]
            [:li
             [:a {:href  (str "https://" domain-name "/")
                  :class "group text-accent space-x-2"}
              [:span {:class "font-bold group-hover:underline group-hover:decoration-2"}
               (or display-name handle)]
              [:span {:class "text-muted"}
               (str " — " domain-name)]]])])))))

;;; ----------------------------------------------------------------------------
;;; Redirect

(defn- redirect-demo
  []
  (ui/card {}
    (ui/card-title (tru "Redirect Demo"))
    (ui/button-primary {:type        "button"
                        :data-action "demo/redirect"}
                       (tru "Go to jcf.dev"))))

(defn redirect-view
  [request]
  (list
   (ui/nav-header request "/redirect")
   (ui/page-center {}
     (redirect-demo))))

;;; ----------------------------------------------------------------------------
;;; Cursors

(defonce !cursors (atom {}))

(defn update-cursor!
  [channel-id x y]
  (swap! !cursors assoc channel-id [x y (System/currentTimeMillis)]))

(defn remove-cursor!
  [channel-id]
  (swap! !cursors dissoc channel-id))

(def ^:private cursor-colors
  ["bg-red-500"    "bg-blue-500"   "bg-green-500"  "bg-yellow-500"
   "bg-purple-500" "bg-pink-500"   "bg-indigo-500" "bg-teal-500"
   "bg-orange-500" "bg-cyan-500"   "bg-lime-500"   "bg-rose-500"])

(defn- cursor-color
  [channel-id]
  (nth cursor-colors (mod (hash channel-id) (count cursor-colors))))

(defn- cursor-styles
  [request cursors]
  [:style {:id "cursor-positions" :nonce (mw/request->nonce request)}
   (html/raw
    (str/join "\n"
              (for [[channel-id [x y _]] cursors]
                (format ".cursor[data-channel=\"%s\"] { left: %dpx; top: %dpx; }"
                        (subs channel-id 0 6) x y))))])

(defn- cursor-label
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
            :class            "relative flex-1 flex flex-col"
            :data-track-mouse "cursor/move"}

      (cursor-styles request cursors)

      (for [[cid _] cursors]
        (cursor-label cid))

      (ui/page-center {}
        (ui/page-title {} (tru "Presence Cursors"))
        (ui/text-muted {:class ["mt-4"]}
          (str (count cursors) " cursor" (when (not= 1 (count cursors)) "s") " active")))])))

;;; ----------------------------------------------------------------------------
;;; Home (realm-based)

(defn- home-view
  [request]
  (let [view-fn (get-in request [:session/realm :realm/view])]
    (assert (fn? view-fn) "No :realm/view in session realm?!")
    (view-fn request)))

(defn- home-layout
  [request & content]
  (let [layout-fn (get-in request [:session/realm :realm/layout])]
    (assert (fn? layout-fn) "No :realm/layout in session realm?!")
    (apply layout-fn request content)))

;;; ----------------------------------------------------------------------------
;;; Module

(def module
  {:name    :bits.module/platform
   :routes  [["/"         (assoc (morph/morphable home-layout home-view)
                                 :bits/page (fn [request]
                                              {:page/title (-> request :session/realm :creator/display-name)}))]
             ["/counter"  (assoc (morph/morphable ui/layout counter-view)
                                 :bits/page {:page/title "Counter"})]
             ["/cursors"  (assoc (morph/morphable ui/layout cursors-view {:on-close remove-cursor!})
                                 :bits/page {:page/title "Cursors"})]
             ["/form"     (assoc (morph/morphable ui/layout form-view)
                                 :bits/page {:page/title "Forms"})]
             ["/redirect" (assoc (morph/morphable ui/layout redirect-view)
                                 :bits/page {:page/title "Redirect"})]]
   :actions {:counter/inc   (fn [_req] (swap! !counter update :count inc))
             :counter/dec   (fn [_req] (swap! !counter update :count dec))
             :demo/redirect (fn [_req] (morph/redirect "https://jcf.dev"))
             :demo/validate {:handler (fn [request]
                                        (let [form        (assoc (::form/form request)
                                                                 ::form/values (get-in request [:parameters :form]))
                                              validation  (form/validate-form form-schema form)
                                              form-status (form/form-status validation)
                                              success?    (and (::form/submitted? form) (not= form-status :bits.form/error))]
                                          (morph/respond (form-view request {:validation validation
                                                                             :success?   success?}))))
                             :params  [[:username :string]
                                       [:email :string]
                                       [:country :string]]}
             :cursor/move   (fn [request]
                              (let [channel-id (get-in request [:params "channel"])
                                    x          (parse-long (get-in request [:params "x"] "0"))
                                    y          (parse-long (get-in request [:params "y"] "0"))]
                                (when (and channel-id x y (< x 10000) (< y 10000))
                                  (update-cursor! channel-id x y))))}})
