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
   [camel-snake-kebab.core :as csk]
   [com.stuartsierra.component :as component]
   [medley.core :as medley]))

;;; ----------------------------------------------------------------------------
;;; Config

(defmacro ^:private env
  [k]
  (let [s (csk/->SCREAMING_SNAKE_CASE_STRING (name k))]
    `(System/getenv ~s)))

(defmacro ^:private env-or
  [k default]
  `(or (env k) ~default))

(defn- parse-pedestal-env
  [s]
  (-> {"dev"  :dev
       "test" :test
       "prod" :prod}
      (get s "prod")
      keyword))

(defn read-config
  []
  (let [database-url (env-or :database-url "jdbc:postgresql://localhost:5432/bits_dev?user=bits&password=please")]
    {:buster        {:resources #{"public/Inter-Bold.woff2"
                                  "public/Inter-Medium.woff2"
                                  "public/Inter-Regular.woff2"
                                  "public/JetBrainsMono-Bold.woff2"
                                  "public/JetBrainsMono-Regular.woff2"
                                  "public/app.css"}}
     :datahike      {:store (datahike/jdbc-url->store database-url)}
     :keymaster     {:argon             {:alg         :argon2id
                                         :iterations  3
                                         :memory      (* 64 1024)
                                         :parallelism 1}
                     :idle-timeout-days 30}
     :pool          {:database-url database-url}
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
   :migrator      (postgres/make-migrator     (:pool config))
   :pool          (postgres/make-pool         (:pool config))
   :reaper        (reaper/make-reaper         (:reaper config))
   :service       (service/make-service       (:service config))
   :session-store (session/make-session-store (:session-store config))})

(def dependencies
  {:pool          [:migrator]
   :reaper        [:pool]
   :service       [:bootstrapper :buster :datahike :keymaster :pool :session-store]
   :session-store [:pool]})

(defn system
  ([]
   (system (read-config)))
  ([config]
   (component/system-using
    (medley/mapply component/system-map (components config))
    dependencies)))
