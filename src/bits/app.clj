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
   [steffan-westcott.clj-otel.api.trace.span :as span]
   [bits.next :as next])
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
   :service {:cookie-name   "bits"
             :cookie-secret (some-> :cookie-secret (env-or "00000000000000000000000000000000") codecs/hex->bytes)
             :http-host     "0.0.0.0"
             :http-port     (env-or :port 3000)}})

;;; ----------------------------------------------------------------------------
;;; System

(defn components
  [config]
  {:bootstrapper (boot/make-bootstrapper (:bootstrapper config))
   :buster       (assets/make-buster     (:buster config))
   :service      (next/make-service      (:service config))})

(def dependencies
  {:service [:bootstrapper :buster]})

(defn system
  ([]
   (system (read-config)))
  ([config]
   (component/system-using
    (medley/mapply component/system-map (components config))
    dependencies)))
