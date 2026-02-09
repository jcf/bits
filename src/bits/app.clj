(ns bits.app
  (:require
   [bits.assets :as assets]
   [bits.boot :as boot]
   [bits.crypto :as crypto]
   [bits.datahike :as datahike]
   [bits.next :as next]
   [bits.next.reaper :as reaper]
   [bits.next.session :as session]
   [bits.postgres :as postgres]
   [bits.service :as service]
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

(defn read-config
  []
  (let [database-url (-> :database-url env normalize-database-url)]
    {:buster        {:resources #{"public/Inter-Bold.woff2"
                                  "public/Inter-Medium.woff2"
                                  "public/Inter-Regular.woff2"
                                  "public/JetBrainsMono-Bold.woff2"
                                  "public/JetBrainsMono-Regular.woff2"
                                  "public/app.css"}}
     :datahike      {:database-url database-url}
     :keymaster     {:argon             {:alg         :argon2id
                                         :iterations  3
                                         :memory      (* 64 1024)
                                         :parallelism 1}
                     :idle-timeout-days 30}
     :postgres      {:database-url database-url}
     :reaper        {:interval-hours 1}
     :service       {:actions          next/actions
                     :cookie-name      "__Host-bits"
                     :csrf-cookie-name "__Host-bits-csrf"
                     :csrf-secret      (env-or :csrf-secret "default-csrf-secret-change-in-prod")
                     :http-host        "0.0.0.0"
                     :http-port        (env-or :port 3000)
                     :max-refresh-ms   50
                     :routes           next/routes
                     :server-name      "Bits"}
     :session-store {:idle-timeout-days 30}}))

;;; ----------------------------------------------------------------------------
;;; System

(defn components
  [config]
  {:bootstrapper  (boot/make-bootstrapper     (:bootstrapper config))
   :buster        (assets/make-buster         (:buster config))
   :datahike      (datahike/make-database     (:datahike config))
   :keymaster     (crypto/make-keymaster      (:keymaster config))
   :migrator      (postgres/make-migrator     (:postgres config))
   :postgres      (postgres/make-postgres     (:postgres config))
   :randomizer    (crypto/make-randomizer     (:randomizer config))
   :reaper        (reaper/make-reaper         (:reaper config))
   :service       (service/make-service       (:service config))
   :session-store (session/make-session-store (:session-store config))})

(def dependencies
  {:postgres      [:migrator :randomizer]
   :reaper        [:postgres]
   :service       [:bootstrapper
                   :buster
                   :datahike
                   :keymaster
                   :postgres
                   :randomizer
                   :session-store]
   :session-store [:postgres]})

(defn system
  ([]
   (system (read-config)))
  ([config]
   (component/system-using
    (medley/mapply component/system-map (components config))
    dependencies)))
