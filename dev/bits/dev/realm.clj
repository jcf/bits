(ns bits.dev.realm
  (:require
   [bits.datahike :as datahike]
   [com.stuartsierra.component.repl :refer [system]]
   [java-time.api :as time]))

(def demo-realms-txes
  [{:db/id       "jcf-domain"
    :domain/name "jcf.bits.page.test"}

   {:db/id       "charlie-domain"
    :domain/name "charlie.bits.page.test"}

   {:db/id       "leather-domain"
    :domain/name "leather.bits.page.test"}

   ;; JCF
   {:tenant/id            (random-uuid)
    :tenant/created-at    (time/java-date)
    :tenant/domains       ["jcf-domain"]
    :creator/handle       "jcf"
    :creator/display-name "James"
    :creator/bio          "Building Bits — censorship-resistant infrastructure for creator sovereignty."}

   ;; Charlie
   {:tenant/id            (random-uuid)
    :tenant/created-at    (time/java-date)
    :tenant/domains       ["charlie-domain"]
    :creator/handle       "charlie"
    :creator/display-name "Charles Montgomery"
    :creator/bio          "Charlie likes treats and special ball."}

   ;; Leather Emporium
   {:tenant/id            (random-uuid)
    :tenant/created-at    (time/java-date)
    :tenant/domains       ["leather-domain"]
    :creator/handle       "leather"
    :creator/display-name "The Leather Emporium"
    :creator/bio          "Purveyors of fine leather goods since 1987."}])

(def localhost-tx
  [{:db/id       "localhost-domain"
    :domain/name "localhost"}

   {:tenant/id            (random-uuid)
    :tenant/created-at    (time/java-date)
    :tenant/domains       ["localhost-domain"]
    :creator/handle       "dev"
    :creator/display-name "Development"
    :creator/bio          "Local development realm"}])

(def seed-txes
  (into localhost-tx demo-realms-txes))

(comment
  (datahike/transact! (:datahike system) seed-txes))
