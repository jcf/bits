(ns bits.dev
  (:require
   [bits.app :as app]
   [bits.asset :as asset]
   [bits.dev.watcher :as watcher]
   [bits.morph :as morph]
   [bits.service :as service]
   [com.stuartsierra.component :as component]
   [com.stuartsierra.component.repl :refer [set-init start stop system]]
   [datomic.api]
   [io.pedestal.log :as log]
   [steffan-westcott.clj-otel.api.trace.span :as span]))

;;; ----------------------------------------------------------------------------
;;; System

(defn- asset-handler
  [watcher events]
  (let [{:keys [buster service]} watcher]
    (log/debug :msg "Recomputing asset hashes...")
    (asset/regurgitate! buster)
    (doseq [event events
            :let  [abs-path (str "/" (:path event))]
            :when (= "app.css" (:path event))]
      (log/debug :msg      "Broadcasting stylesheet update..."
                 :abs-path abs-path)
      (service/broadcast!
       service (morph/stylesheet-event (asset/asset-path buster abs-path))))))

(set-init
 (fn [_system]
   (span/with-span! {:name ::initialize}
     (-> (app/system)
         (assoc ::watcher/watcher (watcher/make-watcher
                                   {:path    "resources/public"
                                    :handler asset-handler}))
         (component/system-using
          (assoc app/dependencies ::watcher/watcher [:buster :service]))))))

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
