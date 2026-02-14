(ns bits.dev.realm
  (:require
   [bits.datomic :as datomic]
   [com.stuartsierra.component.repl :refer [system]]
   [datomic.api :as d]
   [java-time.api :as time]))

(defn seed-txes
  []
  [;; JCF
   {:db/id       "jcf-domain"
    :domain/name "jcf.bits.page.test"}
   {:db/id                 "jcf-post-1"
    :post/id               (random-uuid)
    :post/created-at       (time/java-date)
    :post/text             "Shipped Monero payment research today. Turns out monero-java has JNI bindings that embed directly in the JVM — no external daemon needed."}
   {:db/id                 "jcf-post-2"
    :post/id               (random-uuid)
    :post/created-at       (time/java-date)
    :post/text             "The SSE architecture is working beautifully. When I publish a post, every connected subscriber sees it appear instantly."}
   {:db/id                 "jcf-post-3"
    :post/id               (random-uuid)
    :post/created-at       (time/java-date)
    :post/text             "Deep dive into the Datahike vs Datomic decision and why it matters for self-hosters…"}
   {:tenant/id            (random-uuid)
    :tenant/created-at    (time/java-date)
    :tenant/domains       ["jcf-domain"]
    :creator/posts         ["jcf-post-1" "jcf-post-2" "jcf-post-3"]
    :creator/handle       "jcf"
    :creator/display-name "James"
    :creator/bio          "Building Bits — censorship-resistant infrastructure for creator sovereignty."}

   ;; Charlie
   {:db/id       "charlie-domain"
    :domain/name "charlie.bits.page.test"}
   {:db/id                 "charlie-post-1"
    :post/id               (random-uuid)
    :post/created-at       (time/java-date)
    :post/text             "Found an excellent stick today. Very good stick. 10/10 would fetch again."}
   {:db/id                 "charlie-post-2"
    :post/id               (random-uuid)
    :post/created-at       (time/java-date)
    :post/text             "Special ball update: still special, still ball. No further questions at this time."}
   {:db/id                 "charlie-post-3"
    :post/id               (random-uuid)
    :post/created-at       (time/java-date)
    :post/text             "Conducted extensive research on the couch. Conclusion: very comfortable. Will continue monitoring."}
   {:tenant/id            (random-uuid)
    :tenant/created-at    (time/java-date)
    :tenant/domains       ["charlie-domain"]
    :creator/posts         ["charlie-post-1" "charlie-post-2" "charlie-post-3"]
    :creator/handle       "charlie"
    :creator/display-name "Charles Montgomery"
    :creator/bio          "Charlie likes treats and special ball."}

   ;; Leather Emporium
   {:db/id       "leather-domain"
    :domain/name "leather.bits.page.test"}
   {:db/id                 "leather-post-1"
    :post/id               (random-uuid)
    :post/created-at       (time/java-date)
    :post/text             "New shipment of full-grain Italian leather arrived. The texture on these hides is exceptional."}
   {:db/id                 "leather-post-2"
    :post/id               (random-uuid)
    :post/created-at       (time/java-date)
    :post/text             "Workshop tip: always condition your leather goods every 6 months. Your wallet will thank you in 20 years."}
   {:db/id                 "leather-post-3"
    :post/id               (random-uuid)
    :post/created-at       (time/java-date)
    :post/text             "Behind the scenes look at our saddle-stitching process. Each stitch is made by hand using traditional techniques."}
   {:tenant/id            (random-uuid)
    :tenant/created-at    (time/java-date)
    :tenant/domains       ["leather-domain"]
    :creator/posts         ["leather-post-1" "leather-post-2" "leather-post-3"]
    :creator/handle       "leather"
    :creator/display-name "The Leather Emporium"
    :creator/bio          "Purveyors of fine leather goods since 1987."}

   ;; Milly
   {:db/id       "milly-domain"
    :domain/name "milly.bits.page.test"}
   {:db/id                 "milly-post-1"
    :post/id               (random-uuid)
    :post/created-at       (time/java-date)
    :post/text             "Finally nailed the microfoam today. The secret? Patience and keeping the steam wand at just the right angle. Decaf oat flat white, obviously."}
   {:db/id                 "milly-post-2"
    :post/id               (random-uuid)
    :post/created-at       (time/java-date)
    :post/text             "Customer asked for a caffeine shot. Had to explain (politely) that we're a strictly decaf establishment. The froth is the star here, not the jitters."}
   {:db/id                 "milly-post-3"
    :post/id               (random-uuid)
    :post/created-at       (time/java-date)
    :post/text             "New personal best: 47 seconds from portafilter to perfect rosetta. Would have been faster but got distracted by a squirrel outside."}
   {:tenant/id            (random-uuid)
    :tenant/created-at    (time/java-date)
    :tenant/domains       ["milly-domain"]
    :creator/posts         ["milly-post-1" "milly-post-2" "milly-post-3"]
    :creator/handle       "milly"
    :creator/display-name "Milly"
    :creator/bio          "Aspiring barista. Froth enthusiast. Decaf only — can't have caffeine, what with being a dog."}])

(comment
  (datomic/transact! (:datomic system) (seed-txes))

  (d/q '[:find (pull ?e [*
                         {:creator/posts [*]}
                         {:tenant/domains [:domain/name]}])
         :where [?e :tenant/id]]
       (datomic/db (:datomic system))))
