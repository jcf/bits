(ns bits.cors
  (:require
   [bits.spec]
   [clojure.spec.alpha :as s]
   [clojure.string :as str]
   [io.pedestal.interceptor :refer [interceptor]]
   [io.pedestal.interceptor.chain :as interceptor.chain]
   [medley.core :as medley]
   [ring.util.response :as response]))

;;; --------------------------------------------------------------------------------------------------------------------
;;; Specs

(s/def ::allow-credentials? :bits.service/allow-credentials?)
(s/def ::allowed-headers :bits.service/allowed-headers)
(s/def ::allowed-origins :bits.service/allowed-origins)

(s/def ::cors
  (s/keys :req-un [::allowed-headers
                   ::allowed-origins]
          :opt-un [::allow-credentials?]))

;;; --------------------------------------------------------------------------------------------------------------------

(def ^:private default-allowed-methods
  "GET, OPTIONS")

(def cors-headers
  #{"Access-Control-Allow-Credentials"
    "Access-Control-Allow-Headers"
    "Access-Control-Allow-Methods"
    "Access-Control-Allow-Origin"
    "Access-Control-Max-Age"})

(def ^:private cors-response
  {:status  200
   :headers {"Content-Type" "text/plain"}
   :body    ""})

;;; --------------------------------------------------------------------------------------------------------------------
;;; Helpers

(defn- coll->csv
  [coll]
  (str/join ", " (sort coll)))

;; `allowed-headers` must have been converted to a comma-separated list before calling `grant-access`.
(defn- grant-access
  [response {:keys [allow-credentials? allowed-headers allowed-methods allowed-origin]
             :or   {allow-credentials? true}}]
  {:pre [(s/assert string? allowed-origin)
         (s/assert string? allowed-methods)
         (s/assert (s/nilable string?) allowed-headers)]}
  (let [headers (cond-> {"Access-Control-Allow-Methods"     allowed-methods
                         "Access-Control-Allow-Origin"      allowed-origin
                         "Access-Control-Max-Age"           "7200"}
                  allow-credentials?
                  (assoc "Access-Control-Allow-Credentials" (str allow-credentials?))
                  (some? allowed-headers)
                  (assoc "Access-Control-Allow-Headers" allowed-headers))]
    (update response :headers merge headers)))

;;; --------------------------------------------------------------------------------------------------------------------
;;; Interceptor

(defn make-cors-interceptor
  [cors]
  (let [{:keys [allow-credentials?
                allowed-headers
                allowed-origins]} cors
        allowed-headers           (coll->csv allowed-headers)

        route->allowed-methods
        (->> (:routes cors)
             (group-by first)
             (medley/map-vals (fn [routes]
                                (str/join ", " (into (sorted-set "OPTIONS")
                                                     (comp (map second)
                                                           (map name)
                                                           (map str/upper-case))
                                                     routes)))))]
    (interceptor
     {:name ::cors
      :enter
      (fn enter-cors
        [{:keys [request] :as context}]
        (let [origin     (response/get-header request "origin")
              ;; On some occasions we don't have a `:route` in the `context` so
              ;; can't determine the methods available on the current endpoint.
              ;;
              ;; I need to work out why this is happening, because for any
              ;; interceptor to execute we have to have a corresponding route.
              path       (-> context :route :path)
              preflight? (= :options (:request-method request))]
          (cond
            (and preflight? (contains? allowed-origins origin))
            (let [response (grant-access cors-response
                                         {:allow-credentials? allow-credentials?
                                          :allowed-headers    allowed-headers
                                          :allowed-methods    (get route->allowed-methods path default-allowed-methods)
                                          :allowed-origin     origin})]
              (-> context
                  (assoc :response response)
                  interceptor.chain/terminate))

            preflight?
            (-> context
                (assoc :response cors-response)
                interceptor.chain/terminate)

            :else
            context)))

      :leave
      (fn leave-cors
        [{:keys [request] :as context}]
        (let [origin (response/get-header request "origin")
              path   (-> context :route :path)]
          (cond-> context
            (contains? allowed-origins origin)
            (update :response grant-access {:allow-credentials? allow-credentials?
                                            :allowed-headers    allowed-headers
                                            :allowed-methods    (get route->allowed-methods path default-allowed-methods)
                                            :allowed-origin     origin}))))})))
