(ns bits.dev
  (:require
   [bits.app :as app]
   [com.stuartsierra.component :as component]
   [com.stuartsierra.component.repl :refer [set-init start stop system]]
   [datahike.core]
   [io.pedestal.log :as log]
   [steffan-westcott.clj-otel.api.trace.span :as span]))

;;; ----------------------------------------------------------------------------
;;; System

(set-init
 (fn [_system]
   (span/with-span! {:name ::initialize} (app/system))))

(defn before-refresh
  []
  (try
    (log/debug :msg "Stopping development system...")
    (span/with-span! {:name ::stop-system}
      (stop))
    (catch Exception exception
      (log/warn :in ::before-refresh :exception exception))))

(defn after-refresh
  []
  (try
    (log/debug :msg "Starting development system...")
    (span/with-span! {:name ::start-system}
      (start))
    (catch Exception exception
      (log/warn :in ::after-refresh :exception exception)
      (when-let [system (-> exception ex-data :system)]
        (log/debug :in  ::after-refresh
                   :msg "Stopping broken system...")
        (component/stop-system system)))))
