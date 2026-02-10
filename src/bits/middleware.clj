(ns bits.middleware
  (:require
   [bits.crypto :as crypto]
   [bits.csp :as csp]
   [bits.datahike :as datahike]
   [buddy.core.bytes :as buddy.bytes]
   [clojure.string :as str]
   [datahike.api :as d]
   [io.pedestal.log :as log]
   [ring.util.response :as response]))

;;; ----------------------------------------------------------------------------
;;; State injection

(defn wrap-state
  "Injects service state into request for handlers."
  [handler service]
  (fn [request]
    (handler (assoc request ::state service))))

;;; ----------------------------------------------------------------------------
;;; Accessors

(defn- get-state
  [request k]
  {:post [(some? %)]}
  (get-in request [::state k]))

(defn request->datahike
  [request]
  (get-state request :datahike))

(defn request->keymaster
  [request]
  (get-state request :keymaster))

(defn request->postgres
  [request]
  (get-state request :postgres))

(defn request->randomizer
  [request]
  (get-state request :randomizer))

(defn request->session-store
  [request]
  (get-state request :session-store))

(defn request->state
  [request]
  (::state request))

(defn request->db
  [request]
  {:post [(some? %)]}
  (::db request))

(defn request->nonce
  [request]
  (get-in request [:session :nonce]))

;;; ----------------------------------------------------------------------------
;;; Database

(defn wrap-datahike
  [handler]
  (fn [request]
    (let [datahike (request->datahike request)
          db       (some-> datahike datahike/db)]
      (handler (cond-> request (some? db) (assoc ::db db))))))

;;; ----------------------------------------------------------------------------
;;; Session

(defn wrap-ensure-session
  [handler]
  (fn [request]
    (let [randomizer (request->randomizer request)
          session    (:session request)
          sid        (or (:sid session) (crypto/random-sid randomizer))
          nonce      (or (:nonce session) (crypto/random-nonce randomizer))
          request    (assoc-in request [:session :sid] sid)
          request    (assoc-in request [:session :nonce] nonce)
          response   (handler request)]
      (log/debug :msg "Ensuring session..." :sid sid :nonce nonce)
      (when response
        (update response :session #(merge session {:sid sid :nonce nonce} %))))))

(defn wrap-user
  [handler]
  (fn [request]
    (let [db      (request->db request)
          user-id (get-in request [:session :user/id])
          user    (when (some? user-id)
                    (d/q '[:find (pull ?u [:user/id]) .
                           :in $ ?id
                           :where [?u :user/id ?id]]
                         db
                         user-id))]
      (handler (cond-> request (some? user) (assoc :session/user user))))))

;;; ----------------------------------------------------------------------------
;;; Secure headers

(def ^:private secure-headers
  {"referrer-policy"                   "strict-origin"
   "strict-transport-security"         "max-age=31536000; includeSubdomains"
   "x-content-type-options"            "nosniff"
   "x-download-options"                "noopen"
   "x-frame-options"                   "DENY"
   "x-permitted-cross-domain-policies" "none"
   "x-xss-protection"                  "1; mode=block"})

(defn wrap-secure-headers
  [handler]
  (fn [request]
    (when-let [response (handler request)]
      (let [nonce   (get-in request [:session :nonce])
            policy  (csp/csp-map->str (csp/policy nonce))
            headers (assoc secure-headers "content-security-policy" policy)]
        (update response :headers merge headers)))))

;;; ----------------------------------------------------------------------------
;;; CSRF

(def ^:private safe-methods
  #{:get :head :options})

(defn- sse-request?
  [request]
  (some-> (response/get-header request "accept")
          (str/includes? "text/event-stream")))

(defn- csrf-equals?
  [expected actual]
  (and (some? expected)
       (some? actual)
       (buddy.bytes/equals? (.getBytes ^String expected "UTF-8")
                            (.getBytes ^String actual "UTF-8"))))

(defn wrap-csrf
  [handler {:keys [cookie-name cookie-secure secret]}]
  (fn [request]
    (let [sid            (get-in request [:session :sid])
          token          (crypto/csrf-token secret sid)
          actual         (get-in request [:params "csrf"])
          current-cookie (get-in request [:cookies cookie-name :value])
          safe?          (or (contains? safe-methods (:request-method request))
                             (sse-request? request))
          valid?         (or safe? (csrf-equals? token actual))]
      (log/debug :msg    "Guarding against CSRF..."
                 :sid    sid
                 :safe?  safe?
                 :tokens [token actual])
      (if valid?
        (cond-> (handler (assoc request ::csrf token))
          (not= token current-cookie)
          (assoc-in [:cookies cookie-name] {:value     token
                                            :http-only false
                                            :path      "/"
                                            :same-site :lax
                                            :secure    cookie-secure}))
        {:status 403
         :body   "Invalid CSRF token"}))))
