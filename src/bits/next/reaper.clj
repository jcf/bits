(ns bits.next.reaper
  (:require
   [bits.next.session :as session]
   [com.stuartsierra.component :as component]
   [steffan-westcott.clj-otel.api.trace.span :as span])
  (:import
   (java.util.concurrent Executors ScheduledExecutorService TimeUnit)))

(defrecord Reaper [^ScheduledExecutorService executor
                   interval-hours
                   pool]
  component/Lifecycle
  (start [this]
    (span/with-span! {:name ::start-reaper}
      (let [executor (Executors/newSingleThreadScheduledExecutor)
            task     (fn []
                       (span/with-span! {:name ::reap-sessions}
                         (try
                           (let [deleted (session/delete-expired-sessions! pool)]
                             (span/add-span-data! {:attributes {:deleted deleted}}))
                           (catch Exception e
                             (span/add-exception! e {:escaping? false})))))]
        (.scheduleAtFixedRate executor task
                              0
                              interval-hours
                              TimeUnit/HOURS)
        (assoc this :executor executor))))

  (stop [this]
    (span/with-span! {:name ::stop-reaper}
      (when executor
        (.shutdown executor)
        (when-not (.awaitTermination executor 5 TimeUnit/SECONDS)
          (.shutdownNow executor)))
      (assoc this :executor nil))))

(defn make-reaper
  [{:keys [interval-hours] :or {interval-hours 1}}]
  (map->Reaper {:interval-hours interval-hours}))
