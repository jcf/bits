(ns bits.dev
  (:require
   [bits.app :as app]
   [bits.next :as next]
   [bits.service :as service]
   [clojure.core.async :as a]
   [com.stuartsierra.component.repl :refer [set-init system]]
   [hato.client :as http]))

(set-init (fn [_system] (app/system)))

(comment
  (com.stuartsierra.component.repl/reset)

  (def channels (deref (:channels (:service system))))
  (def send! (:send! (-> channels vals first)))

  (send! (next/title-event "Hello from the server!"))
  (send! (next/title-event "Bits"))

  (do
    (swap! next/!state #(update % :count * 2))
    (a/put! (:refresh-ch (:service system)) :action)))
