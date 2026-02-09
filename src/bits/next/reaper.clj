(ns bits.next.reaper
  (:require
   [bits.auth.rate-limit :as rate-limit]
   [bits.next.session :as session]
   [com.stuartsierra.component :as component]
   [io.pedestal.log :as log]
   [steffan-westcott.clj-otel.api.trace.span :as span])
  (:import
   (java.util.concurrent Executors ScheduledExecutorService TimeUnit)))

(defn purge-sessions!
  [reaper]
  (let [{:keys [postgres]} reaper]
    (span/with-span! {:name ::reap}
      (try
        (let [sessions-deleted (session/delete-expired-sessions! postgres)
              attempts-deleted (rate-limit/delete-old-attempts! postgres)]
          (span/add-span-data! {:attributes {:sessions-deleted sessions-deleted
                                             :attempts-deleted attempts-deleted}})
          {:attempts-deleted attempts-deleted
           :sessions-deleted sessions-deleted})
        (catch Exception ex
          (log/warn :msg "Failed to purge sessions?!" :exception ex)
          (span/add-exception! ex {:escaping? false}))))))

(defrecord Reaper [^ScheduledExecutorService executor
                   interval-hours
                   postgres]
  component/Lifecycle
  (start [this]
    (span/with-span! {:name ::start-reaper}
      (let [executor (Executors/newSingleThreadScheduledExecutor)]
        (.scheduleAtFixedRate executor purge-sessions!
                              0 interval-hours TimeUnit/HOURS)
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
