(ns bits.cli.warmup
  (:require
   [io.pedestal.log :as log]))

(def ^:private targets
  '[bits.app
    bits.cli])

(def spec
  {})

(defn run
  [_component _ctx]
  (doseq [target targets] (require target :reload-all))
  (log/info :msg "Warmup complete."))

(def command
  {:desc "Load classes for AppCDS generation"
   :fn   run
   :spec spec})
