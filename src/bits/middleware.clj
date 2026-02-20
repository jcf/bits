(ns bits.middleware
  (:require
   [bits.asset :as asset]
   [bits.crypto :as crypto]
   [bits.csp :as csp]
   [bits.datomic :as datomic]
   [bits.request :as request]
   [bits.session :as session]
   [buddy.core.bytes :as buddy.bytes]
   [clojure.java.io :as io]
   [clojure.string :as str]
   [datomic.api :as d]
   [ring.util.response :as response]))

;;; ----------------------------------------------------------------------------
;;; State injection

(defn wrap-state
  [handler service]
  (fn [request]
    (handler (assoc request ::state service))))

;;; ----------------------------------------------------------------------------
;;; Accessors

(defn- get-state
  [request k]
  {:post [(some? %)]}
  (get-in request [::state k]))

(defn request->buster
  [request]
  (get-state request :buster))

(defn request->datomic
  [request]
  (get-state request :datomic))

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

(defn request->platform-domain
  [request]
  (get-state request :platform-domain))

(defn request->realms
  [request]
  (get-state request :realms))

(defn request->db
  [request]
  {:post [(some? %)]}
  (::db request))

(defn request->nonce
  [request]
  (get-in request [:session :nonce]))

(defn request->csrf-cookie-name
  [request]
  (get-state request :csrf-cookie-name))

;;; ----------------------------------------------------------------------------
;;; Database

(defn wrap-datomic
  [handler]
  (fn [request]
    (let [database (request->datomic request)
          db       (some-> database datomic/db)]
      (handler (cond-> request (some? db) (assoc ::db db))))))

;;; ----------------------------------------------------------------------------
;;; Session

(defn wrap-session
  [handler]
  (fn [request]
    (let [{:keys [cookie-name
                  session-store]} (request->state request)
          tenant-id               (get-in request [:session/realm :tenant/id])
          sid                     (get-in request [:cookies cookie-name :value])

          persisted
          (when (and tenant-id sid)
            ;; TODO Rename :sid to :session/id
            (session/get-session session-store tenant-id sid))

          request
          (assoc request
                 :session (or (when persisted
                                (assoc (:data persisted)
                                       :sid     sid
                                       :user/id (:user-id persisted)))
                              (session/new-session session-store)))

          response      (handler request)
          {:keys [sid]} (:session response)]
      (cond-> response
        (some? sid) (assoc-in [:cookies cookie-name]
                              {:http-only true
                               :path      "/"
                               :same-site :lax
                               :secure    true
                               :value     sid})))))

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
;;; Realm

(def ^:private realm-by-domain-query
  '[:find (pull ?r [:creator/avatar-url
                    :creator/banner-url
                    :creator/bio
                    :creator/display-name
                    :creator/handle
                    :tenant/id
                    {:creator/links [:link/icon
                                     :link/label
                                     :link/url]}
                    {:creator/posts [:post/created-at
                                     :post/id
                                     :post/image-url
                                     :post/text]}]) .
    :in $ ?domain
    :where
    [?d :domain/name ?domain]
    [?r :tenant/domains ?d]])

(defn- platform?
  [request]
  (or (request/local? request)
      (= (request/domain request) (request->platform-domain request))))

(defn wrap-realm
  [handler realms]
  (fn [request]
    (let [{creator-realm  :realm.type/creator
           platform-realm :realm.type/platform
           unknown-realm  :realm.type/unknown} realms]
      (if (platform? request)
        (handler (assoc request :session/realm platform-realm))
        (let [db     (request->db request)
              domain (request/domain request)
              realm  (or (some->> (d/q realm-by-domain-query db domain)
                                  (merge creator-realm))
                         unknown-realm)]
          (handler (assoc request :session/realm realm)))))))

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

;;; ----------------------------------------------------------------------------
;;; Assets

(defn wrap-assets
  [handler]
  (fn [request]
    (let [buster                (request->buster request)
          {::asset/keys [content-type
                         resource]
           :as          a} (asset/lookup buster request)]
      (if (some? a)
        {:status  200
         :headers {"content-type"  content-type
                   "cache-control" "public, max-age=31536000, immutable"}
         :body    (io/input-stream resource)}
        (handler request)))))
