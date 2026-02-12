(ns bits.dev.remote
  (:require
   [bits.assets :as assets]
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
    (service/refresh! (:service system)))

  (keys (::assets/asset-path->asset (assets/stomach (:buster system))))

  (let [{:keys [buster service]} system
        url                      (assets/asset-path buster "/app.css")]
    {:url       url
     :broadcast (when (some? url)
                  (service/broadcast! service (morph/stylesheet-event url)))}))
