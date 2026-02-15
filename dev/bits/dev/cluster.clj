(ns bits.dev.cluster
  (:require
   [bits.cluster :as cluster]
   [com.stuartsierra.component.repl :refer [system]]
   [java-time.api :as time]))

(comment
  (deref (:view (:cluster system)))
  (cluster/send! (:cluster system) {:event/type :presence/joined
                                    :user/id    (random-uuid)}))
