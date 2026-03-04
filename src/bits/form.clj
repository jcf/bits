(ns bits.form
  (:require
   [bits.cryptex :as cryptex]
   [bits.html :as html]
   [bits.locale :refer [tru]]
   [bits.middleware :as mw]
   [bits.string :as string]
   [bits.tailwind :as tw]
   [clojure.string :as str]
   [malli.core :as m]
   [malli.error :as me]))

;;; ----------------------------------------------------------------------------
;;; Middleware

(defn wrap-form-params
  [handler]
  (fn [request]
    (if-let [form-params (:form-params request)]
      (let [result (reduce-kv
                    (fn [acc k v]
                      (cond
                        (= k "_target")
                        (assoc acc ::target (keyword v))

                        (or (= k "submit") (= k "_submitted"))
                        (assoc acc ::submitted? true)

                        (str/starts-with? k "_unused_")
                        (let [field-name (subs k 8)]
                          (-> acc
                              (assoc-in [:form-params field-name] v)
                              (update ::pristine conj (keyword field-name))))

                        :else
                        (assoc-in acc [:form-params k] v)))
                    {:form-params  {}
                     ::pristine    #{}
                     ::target      nil
                     ::submitted?  false}
                    form-params)
            form   (dissoc result :form-params)]
        (handler (assoc request
                        :form-params (:form-params result)
                        ::raw        form)))
      (handler request))))

;;; ----------------------------------------------------------------------------
;;; Status

(def statuses
  #{::pristine
    ::advisory
    ::error})

;;; ----------------------------------------------------------------------------
;;; Styles

(def ^:private field-classes
  {::advisory {:bg      "bg-amber-400/[0.04]"
               :ring    "ring-1 ring-amber-400/50"
               :shadow  "shadow-[0_0_12px_-3px_rgba(251,191,36,0.15)]"
               :outline "focus-visible:outline-amber-400"}
   ::error    {:bg      "bg-red-500/[0.04]"
               :ring    "ring-2 ring-red-500/60"
               :shadow  "shadow-[0_0_16px_-3px_rgba(239,68,68,0.25)]"
               :outline "focus-visible:outline-red-500"}
   ::pristine {:bg      "bg-white/[0.04]"
               :ring    "ring-1 ring-white/10"
               :shadow  ""
               :outline "focus-visible:outline-accent"}})

(def form-classes
  {::advisory {:bg     "bg-white/[0.02]"
               :ring   "ring-2 ring-amber-400/20"
               :shadow "shadow-[0_0_24px_-6px_rgba(251,191,36,0.08)]"}
   ::error    {:bg     "bg-white/[0.02]"
               :ring   "ring-2 ring-red-500/30"
               :shadow "shadow-[0_0_30px_-6px_rgba(239,68,68,0.15)]"}
   ::pristine {:bg     "bg-white/[0.02]"
               :ring   "ring-1 ring-white/[0.06]"
               :shadow ""}})

(def ^:private hint-classes
  {::advisory "text-amber-400/70"
   ::error    "text-red-400/70"})

(def ^:private submit-base-classes
  ["block" "w-full" "py-2.5" "px-4" "border-none" "rounded-lg"
   "text-sm" "font-medium"
   "cursor-pointer"
   "transition-all" "duration-300" "ease-out"])

(def ^:private submit-classes
  {::error   ["bg-red-500/20" "text-red-400" "ring-2" "ring-red-500/50" "hover:opacity-90"]
   ::success ["bg-accent" "text-surface" "hover:opacity-90"]
   ::idle    ["bg-white/[0.08]" "text-zinc-300" "hover:opacity-80"]})

;;; ----------------------------------------------------------------------------
;;; Validation

(defn- validate-field
  [schema value]
  (when-let [explanation (m/explain schema value)]
    (-> explanation me/humanize first)))

(defn- field-value
  [v]
  (if (cryptex/cryptex? v)
    (cryptex/reveal v)
    v))

(defn form-status
  [validation]
  (if (empty? validation)
    ::pristine
    (let [statuses (mapv (comp :status val) validation)]
      (cond
        (some #{::error} statuses)    ::error
        (some #{::advisory} statuses) ::advisory
        :else                         ::pristine))))

(defn validate-form
  [schema raw]
  (let [{::keys [values pristine target submitted?]} raw]
    (into {}
          (for [[field-kw field-schema] schema
                :let [raw-value (get values field-kw)
                      value     (field-value raw-value)
                      pristine? (contains? pristine field-kw)
                      editing?  (= field-kw target)
                      blank?    (or (nil? value) (= "" value))
                      error     (when-not blank?
                                  (validate-field field-schema value))]]
            [field-kw
             (cond
               pristine?                             {:status ::pristine}
               (and error submitted? (not editing?)) {:status ::error :message error :value value :used true}
               error                                 {:status ::advisory :message error :value value :used true}
               (and blank? submitted?)               {:status ::error :message (tru "Required") :value value :used true}
               (not blank?)                          {:status ::pristine :value value}
               :else                                 {:status ::pristine :value value})]))))

;;; ----------------------------------------------------------------------------
;;; Build

(defn- default-submit
  []
  {:idle    (tru "Submit")
   :error   (tru "Whoops!")
   :success (tru "Success!")})

(defn build
  [request config]
  (let [{:keys [schema submit]
         :or   {schema config}} (if (contains? config :schema)
                                  config
                                  {:schema config})
        submit (merge (default-submit) submit)
        raw        (::raw request)
        values     (update-keys (:form-params request) keyword)
        raw        (assoc raw ::values values)
        validation (validate-form schema raw)
        target     (::target raw)
        submitted? (::submitted? raw)
        editing?   (some? target)
        status     (form-status validation)]
    {:csrf       (::mw/csrf request)
     :editing?   editing?
     :schema     schema
     :status     status
     :submit     submit
     :submitted? submitted?
     :success?   (and submitted? (not editing?) (= status ::pristine))
     :target     target
     :validation validation
     :values     values}))

;;; ----------------------------------------------------------------------------
;;; Modifiers

(defn with-error
  "Mark form as having a server-side error. Optionally provide a custom message."
  ([f]
   (assoc f :status ::error :success? false))
  ([f message]
   (cond-> (assoc f :status ::error :success? false)
     message (assoc-in [:submit :error] message))))

;;; ----------------------------------------------------------------------------
;;; Form element

(def ^:private form-base-classes
  ["transition-opacity" "inert:opacity-50" "inert:cursor-wait"])

(defn form
  [f action & body]
  (let [[attrs & children]        (html/normalize body)
        {:keys [csrf editing?
                status submitted?
                success?]}        f
        {:keys [ring bg shadow]}  (get form-classes status (::pristine form-classes))
        shake?                    (and (= status ::error) (not editing?))]
    (into [:form (-> attrs
                     (assoc :method "post" :action "/action" :novalidate true)
                     (tw/with-defaults [ring bg shadow
                                        "transition-all" "duration-500" "ease-out"
                                        (when shake? "form-shake")])
                     (tw/with-defaults form-base-classes)
                     (cond-> success? (assoc :data-reset true)))
           [:input {:type  "hidden"
                    :name  "action"
                    :value (string/keyword->string action)}]
           [:input {:type  "hidden"
                    :name  "csrf"
                    :value csrf}]
           (when (and submitted? (not success?))
             [:input {:type "hidden" :name "_submitted" :value "true"}])]
          children)))

;;; ----------------------------------------------------------------------------
;;; Action button (outside form context)

(defn action-button
  [action-kw & body]
  (let [[opts children] (html/normalize body)]
    (into [:button (assoc opts
                          :type        "button"
                          :data-action (string/keyword->string action-kw))]
          children)))

;;; ----------------------------------------------------------------------------
;;; Field components

(def ^:private label-classes
  "block text-xs font-medium tracking-wide text-zinc-500 uppercase mb-1.5 pl-0.5")

(def ^:private input-base-classes
  ["w-full" "px-3.5" "py-2.5" "rounded-lg" "text-sm"
   "placeholder:text-zinc-600"
   "outline-1" "outline-offset-1" "outline-transparent"
   "transition-all" "duration-300" "ease-out"
   "text-zinc-200"])

(defn- hint
  [field-id status message]
  [:div {:id    (str field-id "-hint")
         :role  (when (= status ::error) "alert")
         :class (tw/merge-classes ["h-5" "flex" "items-center" "pl-0.5"
                                   "transition-opacity" "duration-300" "ease-out"
                                   (if message "opacity-100" "opacity-0")
                                   (get hint-classes status "text-zinc-500")])}
   [:span {:class "text-xs"} (or message "\u00A0")]])

(defn field
  [f key attrs]
  (let [{:keys [status message value used]} (get-in f [:validation key])
        {:keys [ring bg shadow outline]}    (get field-classes status (::pristine field-classes))
        field-id                            (name key)
        {:keys [label type]
         :or   {type "text"}}               attrs
        password?                           (= type "password")
        success?                            (:success? f)
        hint-id                             (str field-id "-hint")
        input-attrs                         (-> (dissoc attrs :label)
                                                (assoc :id field-id
                                                       :name field-id
                                                       :type type)
                                                (cond->
                                                 (and value (not password?) (not success?)) (assoc :value value)
                                                 (and used (not success?))                  (assoc :data-used "true")
                                                 (= status ::error)                         (assoc :aria-invalid "true")
                                                 message                                    (assoc :aria-describedby hint-id)))]
    [:div
     [:label {:for field-id :class label-classes} label]
     [:div {:class "relative"}
      [:input (tw/with-defaults input-attrs (conj input-base-classes ring bg shadow outline))]]
     (hint field-id status message)]))

;;; ----------------------------------------------------------------------------
;;; Select

(def ^:private chevron-icon
  [:svg {:viewBox     "0 0 16 16"
         :fill        "currentColor"
         :aria-hidden "true"
         :class       ["pointer-events-none" "col-start-1" "row-start-1"
                       "mr-2" "size-5" "self-center" "justify-self-end"
                       "text-zinc-500" "sm:size-4"]}
   [:path {:d         "M4.22 6.22a.75.75 0 0 1 1.06 0L8 8.94l2.72-2.72a.75.75 0 1 1 1.06 1.06l-3.25 3.25a.75.75 0 0 1-1.06 0L4.22 7.28a.75.75 0 0 1 0-1.06Z"
           :clip-rule "evenodd"
           :fill-rule "evenodd"}]])

(def ^:private select-base-classes
  ["col-start-1" "row-start-1"
   "w-full" "appearance-none"
   "px-3.5" "py-2.5" "pr-8" "rounded-lg" "text-sm"
   "outline-1" "outline-offset-1" "outline-transparent"
   "transition-all" "duration-300" "ease-out"
   "text-zinc-200"])

(defn select
  [f key attrs options]
  (let [{:keys [status message value used]} (get-in f [:validation key])
        {:keys [ring bg shadow outline]}    (get field-classes status (::pristine field-classes))
        field-id                            (name key)
        {:keys [label placeholder]}         attrs
        success?                            (:success? f)
        hint-id                             (str field-id "-hint")
        select-attrs                        (-> (dissoc attrs :label :placeholder)
                                                (assoc :id field-id
                                                       :name field-id)
                                                (cond->
                                                 (and value (not success?)) (assoc :value value)
                                                 (and used (not success?))  (assoc :data-used "true")
                                                 (= status ::error)         (assoc :aria-invalid "true")
                                                 message                    (assoc :aria-describedby hint-id)))]
    [:div
     [:label {:for field-id :class label-classes} label]
     [:div {:class "grid grid-cols-1"}
      (into [:select (tw/with-defaults select-attrs (conj select-base-classes ring bg shadow outline))]
            (cons (when placeholder
                    [:option {:value ""} placeholder])
                  options))
      chevron-icon]
     (hint field-id status message)]))

;;; ----------------------------------------------------------------------------
;;; Textarea

(def ^:private textarea-base-classes
  ["w-full" "px-3.5" "py-2.5" "rounded-lg" "text-sm"
   "placeholder:text-zinc-600"
   "outline-1" "outline-offset-1" "outline-transparent"
   "transition-all" "duration-300" "ease-out"
   "text-zinc-200" "resize-none"])

(defn textarea
  [f key attrs]
  (let [{:keys [status message value used]} (get-in f [:validation key])
        {:keys [ring bg shadow outline]}    (get field-classes status (::pristine field-classes))
        field-id                            (name key)
        {:keys [label rows]
         :or   {rows 3}}                    attrs
        success?                            (:success? f)
        hint-id                             (str field-id "-hint")
        textarea-attrs                      (-> (dissoc attrs :label)
                                                (assoc :id field-id
                                                       :name field-id
                                                       :rows rows)
                                                (cond->
                                                 (and value (not success?)) (assoc :value value)
                                                 (and used (not success?))  (assoc :data-used "true")
                                                 (= status ::error)         (assoc :aria-invalid "true")
                                                 message                    (assoc :aria-describedby hint-id)))]
    [:div
     [:label {:for field-id :class label-classes} label]
     [:textarea (tw/with-defaults textarea-attrs (conj textarea-base-classes ring bg shadow outline))]
     (hint field-id status message)]))

;;; ----------------------------------------------------------------------------
;;; Checkbox

(def ^:private checkbox-base-classes
  ["size-4" "rounded" "appearance-none"
   "checked:bg-accent" "checked:border-transparent"
   "transition-all" "duration-300" "ease-out"
   "cursor-pointer"])

(defn checkbox
  [f key attrs]
  (let [{:keys [status message value]}   (get-in f [:validation key])
        {:keys [ring bg shadow]}         (get field-classes status (::pristine field-classes))
        field-id                         (name key)
        {:keys [label]}                  attrs
        success?                         (:success? f)
        checked?                         (and (= value "true") (not success?))
        hint-id                          (str field-id "-hint")]
    [:div
     [:div {:class "flex items-center gap-2"}
      [:input (cond-> {:type  "checkbox"
                       :id    field-id
                       :name  field-id
                       :value "true"
                       :class (tw/merge-classes (conj checkbox-base-classes ring bg shadow))}
                checked?            (assoc :checked true)
                (= status ::error)  (assoc :aria-invalid "true")
                message             (assoc :aria-describedby hint-id))]
      [:label {:for   field-id
               :class "text-sm text-zinc-300 cursor-pointer select-none"}
       label]]
     [:div {:id    hint-id
            :role  (when (= status ::error) "alert")
            :class (tw/merge-classes ["h-5" "flex" "items-center" "pl-6"
                                      "transition-opacity" "duration-300" "ease-out"
                                      (if message "opacity-100" "opacity-0")
                                      (get hint-classes status "text-zinc-500")])}
      [:span {:class "text-xs"} (or message "\u00A0")]]]))

;;; ----------------------------------------------------------------------------
;;; Radio group

(def ^:private radio-base-classes
  ["size-4" "rounded-full" "appearance-none"
   "checked:bg-accent" "checked:border-transparent"
   "transition-all" "duration-300" "ease-out"
   "cursor-pointer"])

(defn radio-group
  [f key attrs options]
  (let [{:keys [status message value used]} (get-in f [:validation key])
        {:keys [ring bg shadow]}            (get field-classes status (::pristine field-classes))
        field-name                          (name key)
        {:keys [label]}                     attrs
        success?                            (:success? f)
        hint-id                             (str field-name "-hint")]
    [:div
     [:div {:class label-classes} label]
     [:div {:class "space-y-2"
            :role  "radiogroup"
            :aria-describedby (when message hint-id)}
      (for [{:keys [option-value option-label]} options
            :let [option-id (str field-name "-" option-value)]]
        [:div {:class "flex items-center gap-2" :key option-value}
         [:input (cond-> {:type  "radio"
                          :id    option-id
                          :name  field-name
                          :value option-value
                          :class (tw/merge-classes (conj radio-base-classes ring bg shadow))}
                   (and (= value option-value) (not success?)) (assoc :checked true)
                   (and used (not success?))                   (assoc :data-used "true")
                   (= status ::error)                          (assoc :aria-invalid "true"))]
         [:label {:for   option-id
                  :class "text-sm text-zinc-300 cursor-pointer select-none"}
          option-label]])]
     (hint field-name status message)]))

;;; ----------------------------------------------------------------------------
;;; Submit button

(defn submit
  ([f] (submit f {}))
  ([f attrs]
   (let [{:keys [status success?]} f
         {:keys [idle error success]} (:submit f)
         label (cond
                 success?           success
                 (= status ::error) error
                 :else              idle)
         state (cond
                 success?           ::success
                 (= status ::error) ::error
                 :else              ::idle)]
     [:button (-> attrs
                  (assoc :type "submit" :name "submit" :value "true")
                  (tw/with-defaults submit-base-classes)
                  (tw/with-defaults (get submit-classes state)))
      label])))
