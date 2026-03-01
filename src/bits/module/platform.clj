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

(def form-schema
  {:text      [:string {:min 3}]
   :email     [:re {:error/message "Invalid email"} #"^[^\s@]+@[^\s@]+\.[^\s@]+$"]
   :password  [:string {:min 8 :error/message "At least 8 characters"}]
   :number    [:re {:error/message "0-100"} #"^(?:[0-9]|[1-9][0-9]|100)$"]
   :date      [:string {:min 1}]
   :time      [:string {:min 1}]
   :url       [:re {:error/message "Invalid URL"} #"^https?://.*"]
   :tel       [:re {:error/message "Invalid phone"} #"^\+?[\d\s-]+$"]
   :search    [:string {:min 1}]
   :textarea  [:string {:min 10 :error/message "At least 10 characters"}]
   :select    [:enum "a" "b" "c"]
   :radio     [:enum "opt1" "opt2" "opt3"]
   :checkbox  [:= "true"]})

(defn- form-demo
  [request {:keys [validation success? submitted? editing?]}]
  (let [validation               (when-not success? validation)
        form-status              (form/form-status validation)
        {:keys [ring bg shadow]} (get form/form-classes form-status)
        error?                   (= form-status :bits.form/error)
        shake?                   (and error? (not editing?))]
    (form/form request :demo/validate
      (cond-> {:class (str "rounded-xl p-6 transition-all duration-500 ease-out "
                           ring " " shadow " " bg " "
                           (when shake? "form-shake"))}
        success? (assoc :data-reset true))

      (when (and submitted? (not success?))
        [:input {:type "hidden" :name "_submitted" :value "true"}])

      [:div {:class "space-y-4"}
       [:h3 {:class "text-sm font-semibold text-zinc-400 uppercase tracking-wide"}
        (tru "Text Inputs")]

       (form/validated-field
        {:name :text :label (tru "Text") :placeholder "Plain text"
         :autocomplete "off" :data-1p-ignore true}
        (get validation :text))

       (form/validated-field
        {:name :email :label (tru "Email") :type "email" :placeholder "you@example.com"
         :autocomplete "off" :data-1p-ignore true}
        (get validation :email))

       (form/validated-field
        {:name :password :label (tru "Password") :type "password" :placeholder "••••••••"
         :autocomplete "off" :data-1p-ignore true}
        (get validation :password))

       (form/validated-field
        {:name :number :label (tru "Number") :type "number" :min 0 :max 100 :placeholder "0-100"}
        (get validation :number))

       (form/validated-field
        {:name :date :label (tru "Date") :type "date"}
        (get validation :date))

       (form/validated-field
        {:name :time :label (tru "Time") :type "time"}
        (get validation :time))

       (form/validated-field
        {:name :url :label (tru "URL") :type "url" :placeholder "https://example.com"}
        (get validation :url))

       (form/validated-field
        {:name :tel :label (tru "Phone") :type "tel" :placeholder "+1 234 567 8900"}
        (get validation :tel))

       (form/validated-field
        {:name :search :label (tru "Search") :type "search" :placeholder "Search..."}
        (get validation :search))]

      [:div {:class "space-y-4 mt-6 pt-6 border-t border-white/10"}
       [:h3 {:class "text-sm font-semibold text-zinc-400 uppercase tracking-wide"}
        (tru "Multi-line")]

       (form/validated-textarea
        {:name :textarea :label (tru "Textarea") :placeholder "Enter at least 10 characters..." :rows 4}
        (get validation :textarea))]

      [:div {:class "space-y-4 mt-6 pt-6 border-t border-white/10"}
       [:h3 {:class "text-sm font-semibold text-zinc-400 uppercase tracking-wide"}
        (tru "Selection")]

       (form/validated-select
        {:name :select :label (tru "Select") :placeholder (tru "Choose an option")}
        [[:option {:value "a"} "Option A"]
         [:option {:value "b"} "Option B"]
         [:option {:value "c"} "Option C"]]
        (get validation :select))

       (form/validated-radio-group
        {:name :radio :label (tru "Radio Group")}
        [{:option-value "opt1" :option-label "Option 1"}
         {:option-value "opt2" :option-label "Option 2"}
         {:option-value "opt3" :option-label "Option 3"}]
        (get validation :radio))

       (form/validated-checkbox
        {:name :checkbox :label (tru "I agree to the terms")}
        (get validation :checkbox))]

      [:div {:class "mt-6"}
       (let [base-classes    "block w-full py-3.5 border-none rounded-lg font-sans text-[0.9375rem] font-semibold cursor-pointer tracking-wide transition-opacity duration-150"
             error-classes   "bg-red-500/20 text-red-400 ring-2 ring-red-500/50 hover:opacity-90"
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
      [:div {:class ["sm:mx-auto" "sm:w-full" "sm:max-w-md"]}
       [:h2 {:class ["mt-10" "text-center" "text-2xl/9" "font-bold"
                     "tracking-tight" "text-primary"]}
        (tru "Forms")]]
      [:div {:class ["mt-10" "sm:mx-auto" "sm:w-full" "sm:max-w-md"]}
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
   :actions {:counter/dec   (fn [_req] (swap! !counter update :count dec))
             :counter/inc   (fn [_req] (swap! !counter update :count inc))
             :cursor/move   (fn [request]
                              (let [channel-id (get-in request [:params "channel"])
                                    x          (parse-long (get-in request [:params "x"] "0"))
                                    y          (parse-long (get-in request [:params "y"] "0"))]
                                (when (and channel-id x y (< x 10000) (< y 10000))
                                  (update-cursor! channel-id x y))))
             :demo/redirect (fn [_req] (morph/redirect "https://jcf.dev"))
             :demo/validate (fn [request]
                              (let [form        (assoc (::form/form request)
                                                       ::form/values (update-keys (:form-params request) keyword))
                                    validation  (form/validate-form form-schema form)
                                    form-status (form/form-status validation)
                                    submitted?  (::form/submitted? form)
                                    editing?    (some? (::form/target form))
                                    success?    (and submitted? (not editing?) (not= form-status :bits.form/error))]
                                (morph/respond (form-view request {:validation  validation
                                                                   :submitted?  submitted?
                                                                   :editing?    editing?
                                                                   :success?    success?}))))}})
