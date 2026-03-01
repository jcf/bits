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
                        ::form       form)))
      (handler request))))

;;; ----------------------------------------------------------------------------
;;; Form generation

(def ^:private default-attrs
  {:method "post"
   :action "/action"
   :class  ["transition-opacity" "inert:opacity-50" "inert:cursor-wait"]})

(defn form
  [request action-kw & body]
  (let [[opts & children] (html/normalize body)
        csrf              (::mw/csrf request)
        attrs             (-> (merge default-attrs (dissoc opts :class))
                              (assoc :class (:class opts))
                              (tw/with-defaults (:class default-attrs)))]
    (into [:form attrs
           [:input {:type  "hidden"
                    :name  "action"
                    :value (string/keyword->string action-kw)}]
           [:input {:type  "hidden"
                    :name  "csrf"
                    :value csrf}]]
          children)))

(defn action-button
  [action-kw & body]
  (let [[opts children] (html/normalize body)]
    (into [:button (assoc opts
                          :type        "button"
                          :data-action (string/keyword->string action-kw))]
          children)))

;;; ----------------------------------------------------------------------------
;;; Status

(def statuses
  #{::pristine
    ::advisory
    ::error})

;;; ----------------------------------------------------------------------------
;;; Styles

(def field-classes
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

(def hint-classes
  {::advisory "text-amber-400/70"
   ::error    "text-red-400/70"})

(defn form-status
  [validation]
  (if (empty? validation)
    ::pristine
    (let [statuses (mapv (comp :status val) validation)]
      (cond
        (some #{::error} statuses)    ::error
        (some #{::advisory} statuses) ::advisory
        :else                         ::pristine))))

;;; ----------------------------------------------------------------------------
;;; Validated field

(defn validated-field
  ([field]
   (validated-field field nil))
  ([field validation]
   (let [{:keys [name label type]}                 field
         {:keys [status message value used]}       validation
         {:keys [ring bg shadow outline]}          (get field-classes status (::pristine field-classes))
         field-id                                  (clojure.core/name name)
         password?                                 (= type "password")
         input-attrs                               (-> (dissoc field :label)
                                                       (assoc :id field-id
                                                              :name field-id)
                                                       (cond-> (not type) (assoc :type "text")))
         base-classes                              ["w-full" "px-3.5" "py-2.5" "rounded-lg" "text-sm"
                                                    "placeholder:text-zinc-600"
                                                    "outline-1" "outline-offset-1" "outline-transparent"
                                                    outline
                                                    "transition-all" "duration-300" "ease-out"
                                                    ring bg shadow
                                                    "text-zinc-200"]]
     [:div
      [:label {:for   field-id
               :class "block text-xs font-medium tracking-wide text-zinc-500 uppercase mb-1.5 pl-0.5"}
       label]
      [:div {:class "relative"}
       [:input
        (cond-> (tw/with-defaults input-attrs base-classes)
          (and value (not password?)) (assoc :value value)
          used                        (assoc :data-used "true"))]]
      [:div {:class (str "h-5 flex items-center pl-0.5 "
                         "transition-opacity duration-300 ease-out "
                         (if message "opacity-100" "opacity-0") " "
                         (get hint-classes status "text-zinc-500"))}
       [:span {:class "text-xs"} (or message "\u00A0")]]])))

;;; ----------------------------------------------------------------------------
;;; Validated select

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

(defn validated-select
  ([field options]
   (validated-select field options nil))
  ([field options validation]
   (let [{:keys [name label placeholder]}    field
         {:keys [status message value used]} validation
         {:keys [ring bg shadow outline]}    (get field-classes status (::pristine field-classes))
         field-id                            (clojure.core/name name)
         select-attrs                        (-> (dissoc field :label :placeholder)
                                                 (assoc :id field-id
                                                        :name field-id))
         base-classes                        ["col-start-1" "row-start-1"
                                              "w-full" "appearance-none"
                                              "px-3.5" "py-2.5" "pr-8" "rounded-lg" "text-sm"
                                              "outline-1" "outline-offset-1" "outline-transparent"
                                              outline
                                              "transition-all" "duration-300" "ease-out"
                                              ring bg shadow
                                              "text-zinc-200"]]
     [:div
      [:label {:for   field-id
               :class "block text-xs font-medium tracking-wide text-zinc-500 uppercase mb-1.5 pl-0.5"}
       label]
      [:div {:class "grid grid-cols-1"}
       (into [:select (cond-> (tw/with-defaults select-attrs base-classes)
                        value (assoc :value value)
                        used  (assoc :data-used "true"))]
             (cons (when placeholder
                     [:option {:value ""} placeholder])
                   options))
       chevron-icon]
      [:div {:class (str "h-5 flex items-center pl-0.5 "
                         "transition-opacity duration-300 ease-out "
                         (if message "opacity-100" "opacity-0") " "
                         (get hint-classes status "text-zinc-500"))}
       [:span {:class "text-xs"} (or message "\u00A0")]]])))

;;; ----------------------------------------------------------------------------
;;; Validated textarea

(defn validated-textarea
  ([field]
   (validated-textarea field nil))
  ([field validation]
   (let [{:keys [name label rows]}               field
         {:keys [status message value used]}     validation
         {:keys [ring bg shadow outline]}        (get field-classes status (::pristine field-classes))
         field-id                                (clojure.core/name name)
         textarea-attrs                          (-> (dissoc field :label)
                                                     (assoc :id field-id
                                                            :name field-id)
                                                     (cond-> (not rows) (assoc :rows 3)))
         base-classes                            ["w-full" "px-3.5" "py-2.5" "rounded-lg" "text-sm"
                                                  "placeholder:text-zinc-600"
                                                  "outline-1" "outline-offset-1" "outline-transparent"
                                                  outline
                                                  "transition-all" "duration-300" "ease-out"
                                                  ring bg shadow
                                                  "text-zinc-200" "resize-none"]]
     [:div
      [:label {:for   field-id
               :class "block text-xs font-medium tracking-wide text-zinc-500 uppercase mb-1.5 pl-0.5"}
       label]
      [:textarea
       (cond-> (tw/with-defaults textarea-attrs base-classes)
         value (assoc :value value)
         used  (assoc :data-used "true"))]
      [:div {:class (str "h-5 flex items-center pl-0.5 "
                         "transition-opacity duration-300 ease-out "
                         (if message "opacity-100" "opacity-0") " "
                         (get hint-classes status "text-zinc-500"))}
       [:span {:class "text-xs"} (or message "\u00A0")]]])))

;;; ----------------------------------------------------------------------------
;;; Validated checkbox

(defn validated-checkbox
  ([field]
   (validated-checkbox field nil))
  ([field validation]
   (let [{:keys [name label]}                field
         {:keys [status message checked]}    validation
         {:keys [ring bg shadow]}            (get field-classes status (::pristine field-classes))
         field-id                            (clojure.core/name name)
         base-classes                        ["size-4" "rounded" "appearance-none"
                                              "checked:bg-accent" "checked:border-transparent"
                                              "transition-all" "duration-300" "ease-out"
                                              ring bg shadow
                                              "cursor-pointer"]]
     [:div
      [:div {:class "flex items-center gap-2"}
       [:input (cond-> {:type  "checkbox"
                        :id    field-id
                        :name  field-id
                        :value "true"
                        :class (tw/merge-classes base-classes)}
                 checked (assoc :checked true))]
       [:label {:for   field-id
                :class "text-sm text-zinc-300 cursor-pointer select-none"}
        label]]
      [:div {:class (str "h-5 flex items-center pl-6 "
                         "transition-opacity duration-300 ease-out "
                         (if message "opacity-100" "opacity-0") " "
                         (get hint-classes status "text-zinc-500"))}
       [:span {:class "text-xs"} (or message "\u00A0")]]])))

;;; ----------------------------------------------------------------------------
;;; Validated radio group

(defn validated-radio-group
  ([field options]
   (validated-radio-group field options nil))
  ([field options validation]
   (let [{:keys [name label]}                field
         {:keys [status message value used]} validation
         {:keys [ring bg shadow]}            (get field-classes status (::pristine field-classes))
         field-name                          (clojure.core/name name)
         base-classes                        ["size-4" "rounded-full" "appearance-none"
                                              "checked:bg-accent" "checked:border-transparent"
                                              "transition-all" "duration-300" "ease-out"
                                              ring bg shadow
                                              "cursor-pointer"]]
     [:div
      [:div {:class "block text-xs font-medium tracking-wide text-zinc-500 uppercase mb-1.5 pl-0.5"}
       label]
      [:div {:class "space-y-2"}
       (for [{:keys [option-value option-label]} options
             :let [option-id (str field-name "-" option-value)]]
         [:div {:class "flex items-center gap-2" :key option-value}
          [:input (cond-> {:type  "radio"
                           :id    option-id
                           :name  field-name
                           :value option-value
                           :class (tw/merge-classes base-classes)}
                    (= value option-value) (assoc :checked true)
                    used                   (assoc :data-used "true"))]
          [:label {:for   option-id
                   :class "text-sm text-zinc-300 cursor-pointer select-none"}
           option-label]])]
      [:div {:class (str "h-5 flex items-center pl-0.5 "
                         "transition-opacity duration-300 ease-out "
                         (if message "opacity-100" "opacity-0") " "
                         (get hint-classes status "text-zinc-500"))}
       [:span {:class "text-xs"} (or message "\u00A0")]]])))

;;; ----------------------------------------------------------------------------
;;; Validation

(defn validate-field
  [schema value]
  (when-let [explanation (m/explain schema value)]
    (-> explanation me/humanize first)))

(defn- field-value
  [v]
  (if (cryptex/cryptex? v)
    (cryptex/reveal v)
    v))

(defn validate-form
  [schema form]
  (let [{::keys [values pristine target submitted?]} form]
    (into {}
          (for [[field-kw field-schema] schema
                :let [raw-value (get values field-kw)
                      value     (field-value raw-value)
                      pristine? (contains? pristine field-kw)
                      editing?  (= field-kw target)
                      blank?    (or (and (string? value) (str/blank? value)) (nil? value))
                      error     (when-not blank?
                                  (validate-field field-schema value))]]
            [field-kw
             (cond
               pristine?                             {:status ::pristine}
               (and error submitted? (not editing?)) {:status ::error :message error :value value :used true}
               error                                 {:status ::advisory :message error :value value :used true}
               (and blank? submitted?)               {:status ::error :message (tru "Required") :used true}
               (not blank?)                          {:status ::pristine :value value :used true}
               :else                                 {:status ::pristine :used true})]))))
