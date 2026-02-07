(ns bits.dev
  (:require
   [bits.app :as app]
   [bits.next :as next]
   [bits.service :as service]
   [clojure.core.async :as a]
   [com.stuartsierra.component :as component]
   [com.stuartsierra.component.repl :refer [set-init start stop system]]
   [hato.client :as http]
   [io.pedestal.log :as log]
   [steffan-westcott.clj-otel.api.trace.span :as span]))

(set-init (fn [_system] (app/system)))

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

(set-init
 (fn [_system]
   (span/with-span! {:name ::initialize} (app/system))))

(comment
  (com.stuartsierra.component.repl/reset)
  (com.stuartsierra.component.repl/stop)

  (next/stats (:service system))

  (def channels (deref (:channels (:service system))))
  (def send! (:send! (-> channels vals first)))

  (send! (next/title-event "Hello from the server!"))
  (send! (next/title-event "Bits"))

  (do
    (reset! bits.next/!cursors {})
    (next/refresh! (:service system)))

  (do
    (swap! next/!state #(update % :count * 2))
    (next/refresh!)))
