(ns bits.test.hooks
  (:require
   [bits.test.app :as t]
   [com.stuartsierra.component :as component]
   [io.pedestal.log :as log]))

(defn pre-suite
  [suite _test-plan]
  (log/info :msg "Warming system before test suite...")
  (-> (t/system) t/must-start-system component/stop-system)
  suite)
