(ns bits.dev.cluster
  (:require
   [bits.cluster :as cluster]
   [com.stuartsierra.component.repl :refer [system]]))

(comment
  (deref (:view (:cluster system)))
  (cluster/send! (:cluster system) {:event/type :presence/joined
                                    :user/id    (random-uuid)}))
