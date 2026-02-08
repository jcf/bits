(ns bits.main
  (:require
   [bits.app :as app]
   [com.stuartsierra.component :as component]
   [io.pedestal.log :as log])
  (:gen-class))

(defn -main
  [& args]
  (when (some #{"--dry-run"} args)
    (log/info :msg "Dry run — exiting")
    (System/exit 0))
  (let [system (-> (app/system) component/start)]
    (log/info :msg "Bits started")
    (.addShutdownHook
     (Runtime/getRuntime)
     (Thread.
      ^Runnable
      (fn []
        (log/info :msg "Shutting down...")
        (component/stop system)
        (log/info :msg "Goodbye"))))
    @(promise)))
