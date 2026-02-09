(ns bits.dev.remote
  (:require
   [bits.morph :as morph]
   [bits.next :as next]
   [bits.service :as service]
   [com.stuartsierra.component.repl :refer [system]]
   [datahike.core]))

(comment
  (com.stuartsierra.component.repl/reset)
  (com.stuartsierra.component.repl/stop)

  (service/stats (:service system))

  (def channels (deref (:channels (:service system))))
  (def send! (:send! (-> channels vals first)))

  (send! (morph/title-event "Hello from the server!"))
  (send! (morph/title-event "Bits"))

  (do
    (reset! next/!cursors {})
    (service/refresh! (:service system)))

  (do
    (swap! next/!state update :count * 2)
    (service/refresh! (:service system))))
