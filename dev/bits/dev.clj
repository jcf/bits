(ns bits.dev
  (:require
   [bits.app :as app]
   [bits.service :as service]
   [com.stuartsierra.component.repl :refer [set-init]]
   [hato.client :as http]))

(set-init (fn [_system] (app/system)))

(comment
  (com.stuartsierra.component.repl/reset))
