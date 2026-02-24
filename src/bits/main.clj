(ns bits.main
  (:require
   [bits.app :as app]
   [com.stuartsierra.component :as component]
   [io.pedestal.log :as log])
  (:gen-class))

(defn -main
  [& args]
  ;; --warmup: Exit after class loading. Used during container build to generate
  ;; the AppCDS class list for faster subsequent JVM startups.
  (when (some #{"--warmup"} args)
    (require 'bits.app :reload-all)
    (log/info :msg "Warmup complete. Exiting.")
    (System/exit 0))

  (component/start (app/system))
  (log/info :msg "Your Bits are ready.")
  @(promise))
