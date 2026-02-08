(ns bits.app
  (:require
   [bits.assets :as assets]
   [bits.boot :as boot]
   [bits.datahike :as datahike]
   [bits.next :as next]
   [bits.next.reaper :as reaper]
   [bits.next.session :as session]
   [bits.postgres :as postgres]
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
     :pool          {:database-url database-url}
     :reaper        {:interval-hours 1}
     :service       {:cookie-name      "__Host-bits"
                     :csrf-cookie-name "__Host-bits-csrf"
                     :csrf-secret      (env-or :csrf-secret "default-csrf-secret-change-in-prod")
                     :http-host        "0.0.0.0"
                     :http-port        (env-or :port 3000)
                     :server-name      "bits"}
     :session-store {:idle-timeout-days 30}}))

;;; ----------------------------------------------------------------------------
;;; System

(defn components
  [config]
  {:bootstrapper  (boot/make-bootstrapper     (:bootstrapper config))
   :buster        (assets/make-buster         (:buster config))
   :datahike      (datahike/make-database     (:datahike config))
   :migrator      (postgres/make-migrator     (:pool config))
   :pool          (postgres/make-pool         (:pool config))
   :reaper        (reaper/make-reaper         (:reaper config))
   :service       (next/make-service          (:service config))
   :session-store (session/make-session-store (:session-store config))})

(def dependencies
  {:pool          [:migrator]
   :reaper        [:pool]
   :service       {:bootstrapper  :bootstrapper
                   :buster        :buster
                   :database      :datahike
                   :pool          :pool
                   :session-store :session-store}
   :session-store [:pool]})

(defn system
  ([]
   (system (read-config)))
  ([config]
   (component/system-using
    (medley/mapply component/system-map (components config))
    dependencies)))
