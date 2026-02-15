(ns bits.app
  (:require
   [bits.asset :as asset]
   [bits.auth.rate-limit :as rate-limit]
   [bits.boot :as boot]
   [bits.crypto :as crypto]
   [bits.datomic :as datomic]
   [bits.next :as next]
   [bits.postgres :as postgres]
   [bits.reaper :as reaper]
   [bits.service :as service]
   [bits.session :as session]
   [bits.spec]
   [bits.string :as string]
   [camel-snake-kebab.core :as csk]
   [com.stuartsierra.component :as component]
   [lambdaisland.uri :as uri]
   [medley.core :as medley]))

;;; ----------------------------------------------------------------------------
;;; Config

(defmacro ^:private env
  [k]
  (let [s (csk/->SCREAMING_SNAKE_CASE_STRING (name k))]
    `(System/getenv ~s)))

(defmacro ^:private env-or
  [k default]
  `(or (env ~k) ~default))

(defn- normalize-database-url
  ([s]
   (normalize-database-url s "postgresql"))
  ([s adapter]
   (let [url                     (uri/uri (string/remove-prefix s "jdbc:"))
         query                   (uri/query-string->map (:query url))
         {:keys [user password]} (merge (select-keys url [:user :password]) query)]
     (str (assoc url
                 :password nil
                 :query    (uri/map->query-string {:user user :password password})
                 :scheme   (str "jdbc:" adapter)
                 :user     nil)))))

;; TODO Use Malli (or clojure.spec) to coerce and parse/validate configuration.
(defn read-config
  []
  (let [database-url (-> :database-url env normalize-database-url)]
    {:buster        {:resources #{"public/apple-touch-icon.png"
                                  "public/app.css"
                                  "public/bits.js"
                                  "public/DMSans.woff2"
                                  "public/DMSerifDisplay.woff2"
                                  "public/favicon.ico"
                                  "public/favicon.svg"
                                  "public/idiomorph@0.7.4.min.js"
                                  "public/JetBrainsMono.woff2"
                                  "public/logo.svg"}}
     :datomic       {:uri (env :datomic-uri)}
     :keymaster     {:argon {:alg         :argon2id
                             :iterations  3
                             :memory      (* 64 1024)
                             :parallelism 1}}
     :postgres      {:database-url database-url}
     :rate-limiter  {:email-window-minutes 15
                     :email-max-attempts   5
                     :ip-window-minutes    15
                     :ip-max-attempts      20}
     :reaper        {:interval-hours 1}
     :service       {:actions          next/actions
                     :cookie-name      "__Host-bits"
                     :cookie-secure    true
                     :csrf-cookie-name "__Host-bits-csrf"
                     :csrf-secret      (env-or :csrf-secret "default-csrf-secret-change-in-prod")
                     :http-host        "0.0.0.0"
                     :http-port        (parse-long (env-or :port "3000"))
                     :max-refresh-ms   50
                     :platform-domain  (env :platform-domain)
                     :realms           next/realms
                     :routes           next/routes
                     :server-name      "Bits"
                     :sse-reconnect-ms (parse-long (env-or :sse-reconnect-ms "1000"))}
     :session-store {:idle-timeout-days 30}}))

;;; ----------------------------------------------------------------------------
;;; System

(defn components
  [config]
  {:bootstrapper  (boot/make-bootstrapper     (:bootstrapper config))
   :buster        (asset/make-buster          (:buster config))
   :datomic       (datomic/make-datomic       (:datomic config))
   :keymaster     (crypto/make-keymaster      (:keymaster config))
   :migrator      (postgres/make-migrator     (:postgres config))
   :postgres      (postgres/make-postgres     (:postgres config))
   :randomizer    (crypto/make-randomizer     (:randomizer config))
   :rate-limiter  (rate-limit/make-limiter    (:rate-limiter config))
   :reaper        (reaper/make-reaper         (:reaper config))
   :service       (service/make-service       (:service config))
   :session-store (session/make-session-store (:session-store config))})

(def dependencies
  {:postgres      [:migrator :randomizer]
   :rate-limiter  [:postgres]
   :reaper        [:postgres :session-store]
   :service       [:bootstrapper
                   :buster
                   :datomic
                   :keymaster
                   :postgres
                   :randomizer
                   :rate-limiter
                   :session-store]
   :session-store [:postgres :randomizer]})

(defn system
  ([]
   (system (read-config)))
  ([config]
   (component/system-using
    (medley/mapply component/system-map (components config))
    dependencies)))
