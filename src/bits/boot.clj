(ns bits.boot
  (:require
   [com.stuartsierra.component :as component]
   [io.pedestal.log :as log]
   [steffan-westcott.clj-otel.api.trace.span :as span])
  (:import
   (java.security Security)))

(defrecord Bootstrapper []
  component/Lifecycle
  (start [this]
    (span/with-span! {:name ::start-bootstrapper}
      ;; IP addresses can and will change. Let's not get left behind.
      (Security/setProperty "networkaddress.cache.ttl" (str 60 #_seconds))
      (Thread/setDefaultUncaughtExceptionHandler
       (reify Thread$UncaughtExceptionHandler
         (uncaughtException [_ thread exception]
           (log/error :msg       "Uncaught exception!?"
                      :exception exception
                      :thread    (.getName thread))))))
    this)
  (stop [this]
    (span/with-span! {:name ::stop-bootstrapper}
      this)))

(defn make-bootstrapper
  [config]
  (map->Bootstrapper config))

(defmethod print-method Bootstrapper
  [_ ^java.io.Writer w]
  (.write w "#<Bootstrapper>"))
