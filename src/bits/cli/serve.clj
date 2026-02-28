(ns bits.cli.serve
  (:require
   [bits.app :as app]
   [com.stuartsierra.component :as component]
   [io.pedestal.log :as log]))

(def spec
  {})

(defn run
  [_component _ctx]
  (component/start (app/system))
  (log/info :msg "Your Bits are ready.")
  @(promise))

(def command
  {:desc "Start the HTTP server"
   :fn   run
   :spec spec})
