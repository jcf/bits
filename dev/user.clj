(ns user
  (:require
   [clojure.spec.alpha :as s]
   [clojure.tools.namespace.repl]
   [com.stuartsierra.component.user-helpers]
   [io.pedestal.log :as log]))

(s/check-asserts true)
(clojure.tools.namespace.repl/set-refresh-dirs "dev" "src" "test")
(com.stuartsierra.component.user-helpers/set-dev-ns 'bits.dev)

(if-let [install! (requiring-resolve 'bits.test.telemetry/install!)]
  (install!)
  (log/warn :msg "Test resources not on classpath; test telemetry unavailable."))
