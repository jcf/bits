(ns bits.interceptor
  (:require
   [bits.assets :as assets]
   [bits.cryptex :as cryptex]
   [bits.response]
   [bits.session :as session]
   [clojure.string :as str]
   [io.pedestal.http.response :as http.response]
   [io.pedestal.interceptor :refer [interceptor]]
   [io.pedestal.interceptor.chain :as chain]
   [io.pedestal.interceptor.error :as error]
   [io.pedestal.log :as log]
   [lambdaisland.uri :as uri]
   [medley.core :as medley]
   [ring.util.request :as request]
   [ring.util.response :as response]
   [ring.util.time]
   [steffan-westcott.clj-otel.api.trace.span :as span]))

;;; --------------------------------------------------------------------------------------------------------------------
;;; Request -> State

(def ^:private state-key ::state)

(defn- view
  [& ks]
  (fn [m]
    {:post [(some? %)]}
    (get-in m ks)))

(def context->buster  (view :request state-key :buster))
(def request->buster  (view state-key :buster))

;;; --------------------------------------------------------------------------------------------------------------------
;;; Chain

(defn prepend
  "Prepend an `interceptor` to the given interceptor `chain`. Note, Pedestal
  expects `chain` to be a vector so we both require and return a vector."
  [chain interceptor]
  {:pre [(vector? chain)]}
  (into [interceptor] chain))

(defn terminate
  [context response]
  (-> context (assoc :response response) chain/terminate))

;;; --------------------------------------------------------------------------------------------------------------------
;;; Request

(defn make-request-interceptor
  "Associate value `v` into the context's request under key `k`."
  [k v]
  (interceptor {:name ::make-request :enter #(assoc-in % [:request k] v)}))

(defn make-state-interceptor
  [state]
  (make-request-interceptor state-key state))

;;; ----------------------------------------------------------------------------
;;; Error

(def error-interceptor
  (error/error-dispatch
   [{:keys [request response] :as context} exception]
   :else
   (span/with-span! {:name ::error-dispatch-catch-all}
     (let [data            (ex-data exception)
           pedestal-keys   #{:exception
                             :exception-type
                             :execution-id
                             :interceptor
                             :stage}
           {:keys [execution-id
                   interceptor
                   stage]} data
           more-data       (apply dissoc (ex-data exception) pedestal-keys)]
       (span/add-span-data! {:status     {:code :error}
                             :attributes more-data})
       (log/error :msg            "Interceptor exception?!"
                  :interceptor    interceptor
                  :stage          stage
                  :execution-id   execution-id
                  :request-method (:request-method request)
                  :uri            (:uri request)
                  :ex-data        more-data
                  :exception      exception)
       (assoc context
              ::chain/error nil
              :response     bits.response/internal-server-error-response)))))

;;; ----------------------------------------------------------------------------
;;; Not found

(defn htmx-request?
  [request]
  (= "true" (response/get-header request "hx-request")))

(defn htmx-boosted-request?
  [request]
  (not= "true" (response/get-header request "hx-boosted")))

(defn- not-found-response
  [_context]
  bits.response/not-found-response)

(def not-found-interceptor
  (interceptor
   {:name ::not-found
    :leave
    (fn leave-not-found
      [{:keys [request response] :as context}]
      (span/with-span! {:name ::leave-not-found}
        (if (http.response/response? response)
          context
          (let [response (if (and (htmx-request? request)
                                  (not (htmx-boosted-request? request)))
                           {:status  404
                            :headers {"Content-Type" "text/plain"}
                            :body    "Not found.\n"}
                           (not-found-response context))]
            (assoc context :response response)))))}))

;;; ----------------------------------------------------------------------------
;;; Proxy headers

(defn- first-forwarded-for
  [s]
  (->> (str/split s #"," 2)
       (mapv str/trim)
       first))

(def proxy-headers-interceptor
  (interceptor
   {:name ::proxy-headers-interceptor
    :enter
    (fn enter-proxy-headers
      [{:keys [request] :as context}]
      (span/with-span! {:name ::enter-proxy-headers}
        (if-let [forwarded-for (response/get-header request "x-forwarded-for")]
          (let [remote-addr (first-forwarded-for forwarded-for)]
            (assoc-in context [:request :remote-addr] remote-addr))
          context)))}))

(comment
  (first-forwarded-for "81.131.51.186, 162.158.86.20"))

;;; ----------------------------------------------------------------------------
;;; Referrer policy

(defn make-referrer-policy-interceptor
  "Adds a `Referrer-Policy` header with the given `policy` to any response
  without an existing `Referrer-Policy` header.

  Unlike Pedestal, header names are treated as case-insensitive. Defaults to
  `strict-origin`.

  https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Referrer-Policy"
  ([]
   (make-referrer-policy-interceptor "strict-origin"))
  ([policy]
   (interceptor
    {:name ::referrer-policy
     :leave
     (fn leave-referrer-policy
       [{:keys [response] :as context}]
       (span/with-span! {:name ::leave-referer-policy}
         (cond-> context
           (http.response/response? response)
           (update :response response/update-header "referrer-policy" #(or % policy)))))})))

;;; ----------------------------------------------------------------------------
;;; Assets

(def reload-assets-interceptor
  (interceptor
   {:name ::reload-assets
    :enter
    (fn enter-reload-assets
      [{:keys [request] :as context}]
      (span/with-span! {:name ::reload-assets}
        (log/trace :msg            "Regurgitating assets..."
                   :request-method (:request-method request)
                   :uri            (:uri request))
        (update-in context [:request state-key :buster] assets/regurgitate)))}))

(def asset-interceptor
  (interceptor
   {:name ::asset
    :enter
    (fn enter-asset
      [{:keys [request] :as context}]
      (span/with-span! {:name ::enter-asset}
        (let [buster (context->buster context)]
          (if-let [{::assets/keys [content-type resource]} (assets/lookup buster request)]
            (do
              (log/trace :in ::asset :content-type content-type :resource resource)
              (let [{:keys [content content-length last-modified]} (response/resource-data resource)]
                (terminate context {:status  200
                                    :headers {"Cache-Control"  "public, immutable, max-age=31536000"
                                              "Content-Type"   content-type
                                              "Content-Length" (str content-length)
                                              "Last-Modified"  (ring.util.time/format-date last-modified)}
                                    :body    content})))
            context))))}))

;;; ----------------------------------------------------------------------------
;;; Canonical redirect

(defn- canonical-url
  [url hostname]
  (span/with-span! {:name ::canonical-url}
    (-> url uri/uri (assoc :host hostname) str)))

(comment
  (canonical-url "https://example.com/foo?bar=baz#quux" "canonical.biz"))

(defn make-canonical-redirect-interceptor
  [canonical-host]
  {:pre [(string? canonical-host)]}
  (interceptor
   {:name ::canonical-redirect
    :enter
    (fn enter-canonical-redirect
      [{:keys [request] :as context}]
      (span/with-span! {:name ::enter-canonical-redirect}
        (let [server-name (:server-name request)]
          (if (.equalsIgnoreCase canonical-host server-name)
            context
            (let [_        (span/add-span-data! {:status     {:code :ok}
                                                 :attributes {:canonical-host canonical-host
                                                              :server-name    server-name}})
                  location (canonical-url (request/request-url request) canonical-host)]
              (log/debug :msg            "Redirecting to canonical domain..."
                         :server-name    server-name
                         :canonical-host canonical-host
                         :location       location)
              (terminate context (response/redirect location 301)))))))}))

;;; ------------------------------------------------------------------------------------------------------------------
;;; Cryptex

(defn protect-params-interceptor
  [params]
  (interceptor
   {:name ::protect-params
    :enter
    (fn [context]
      (reduce (fn [ctx path]
                (medley/update-existing-in ctx (into [:request] path) cryptex/cryptex))
              context params))}))
