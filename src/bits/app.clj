(ns bits.app
  (:require
   [bits.assets :as assets]
   [bits.boot :as boot]
   [bits.service :as service]
   [bits.spec]
   [buddy.core.codecs :as codecs]
   [camel-snake-kebab.core :as csk]
   [com.stuartsierra.component :as component]
   [io.pedestal.log :as log]
   [medley.core :as medley]
   [steffan-westcott.clj-otel.api.trace.span :as span])
  (:import
   (java.security Security)))

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
  {:buster  {:resources #{"public/Inter-Bold.woff2"
                          "public/Inter-Medium.woff2"
                          "public/Inter-Regular.woff2"
                          "public/JetBrainsMono-Bold.woff2"
                          "public/JetBrainsMono-Regular.woff2"
                          "public/app.css"}}
   :service {:allow-credentials? true
             :allowed-headers    #{"Authorization"
                                   "Content-Type"
                                   "Accept"
                                   "Origin"}
             :allowed-origins    #{"http://localhost:3000"}
             :canonical-host     (env-or :canonical-host "localhost")
             :commit-id          (env-or :commit-id "unset")
             :cookie-name        "s"
             :cookie-secret      (some-> :cookie-secret (env-or "00000000000000000000000000000000") codecs/hex->bytes)
             :diff-middleware?   false
             :enable-analytics?  (or (-> :enable-analytics (env-or "false") parse-boolean) false)
             :env                (parse-pedestal-env (env :pedestal-env))
             :http-host          "0.0.0.0"
             :http-port          (or (some-> (env :port) parse-long) 3000)
             :join?              false
             :name               (env-or :service-name "bits")
             :origin             (env-or :origin "http://localhost:3000")
             :reload-analytics?  (or (-> :reload-analytics (env-or "false") parse-boolean) false)
             :server-header      "Bits"}})

;;; ----------------------------------------------------------------------------
;;; System

(defn components
  [config]
  {:bootstrapper (boot/make-bootstrapper (:bootstrapper config))
   :buster       (assets/make-buster     (:buster config))
   :service      (service/make-service   (:service config))})

(def dependencies
  {:service [:bootstrapper :buster]})

(defn system
  ([]
   (system (read-config)))
  ([config]
   (component/system-using
    (medley/mapply component/system-map (components config))
    dependencies)))
