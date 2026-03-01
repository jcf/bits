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

                        (= k "submit")
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
  {::advisory {:bg     "bg-amber-400/[0.01]"
               :ring   "ring-1 ring-amber-400/20"
               :shadow "shadow-[0_0_24px_-6px_rgba(251,191,36,0.08)]"}
   ::error    {:bg     "bg-red-500/[0.02]"
               :ring   "ring-2 ring-red-500/30"
               :shadow "shadow-[0_0_30px_-6px_rgba(239,68,68,0.15)]"}
   ::pristine {:bg     "bg-white/[0.02]"
               :ring   "ring-1 ring-white/[0.06]"
               :shadow ""}})

(def hint-classes
  {::advisory "text-amber-400/80"
   ::error    "text-red-400"})

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
         password?                                 (= type "password")
         input-attrs                               (-> (dissoc field :label)
                                                       (assoc :name (clojure.core/name name))
                                                       (cond-> (not type) (assoc :type "text")))
         base-classes                              ["w-full" "px-3.5" "py-2.5" "rounded-lg" "text-sm"
                                                    "placeholder:text-zinc-600"
                                                    "outline-1" "outline-offset-1" "outline-transparent"
                                                    outline
                                                    "transition-all" "duration-300" "ease-out"
                                                    ring bg shadow
                                                    "text-zinc-200"]]
     [:div
      [:label {:class "block text-xs font-medium tracking-wide text-zinc-500 uppercase mb-1.5 pl-0.5"}
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
                      blank?    (or (nil? value) (str/blank? value))
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
