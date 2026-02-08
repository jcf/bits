(ns bits.main
  (:require
   [bits.app :as app]
   [com.stuartsierra.component :as component]
   [io.pedestal.log :as log])
  (:gen-class))

(defn -main
  [& _args]
  (let [system (-> (app/system)
                   component/start)]
    (.addShutdownHook
     (Runtime/getRuntime)
     (Thread.
      ^Runnable
      (fn []
        (log/info :msg "Shutting down...")
        (component/stop system))))
    (log/info :msg "System started.")
    @(promise)))
