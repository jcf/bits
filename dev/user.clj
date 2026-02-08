(ns user
  (:require
   [clojure.spec.alpha :as s]
   [clojure.tools.namespace.repl]
   [com.stuartsierra.component.user-helpers]
   [taoensso.timbre :as timbre]))

(s/check-asserts true)
(clojure.tools.namespace.repl/set-refresh-dirs "dev" "src" "test")
(com.stuartsierra.component.user-helpers/set-dev-ns 'bits.dev)

(timbre/merge-config!
 {:min-level [[#{"datahike.*"} :warn]
              [#{"*"}          :debug]]})
