(ns bits.dev.realm
  (:require
   [bits.datomic :as datomic]
   [bits.ledger :as ledger]
   [com.stuartsierra.component.repl :refer [system]]
   [datomic.api :as d]
   [hasch.core :as hasch]
   [java-time.api :as time]))

;;; ----------------------------------------------------------------------------
;;; Seeder

(defn make-seeder
  [instant]
  {:instant instant})

;;; ----------------------------------------------------------------------------
;;; Helpers

(defn- identify
  [m k]
  (assoc m k (hasch/uuid [k (dissoc m k)])))

(defn- product-tx
  [seeder m]
  (let [instant (:instant seeder)
        tempid  (:db/id m)]
    (-> {:db/id               tempid
         :product/title       (:product/title m)
         :product/description (:product/description m)
         :product/status      :product.status/active
         :product/position    (:product/position m)
         :product/created-at  instant
         :product/variants    [{:variant/id         (hasch/uuid [tempid :variant/id])
                                :variant/name       "Digital Download"
                                :variant/type       :variant.type/digital
                                :variant/active?    true
                                :variant/created-at instant
                                :variant/sku        {:sku/code (:sku/code m)}
                                :variant/price      {:money/amount   (:money/amount m)
                                                     :money/currency :currency/GBP}}]}
        (identify :product/id))))

(defn- accounts-tx
  [seeder handle tempids]
  (let [instant  (:instant seeder)
        accounts (ledger/default-accounts-txes :currency/GBP)]
    (map-indexed (fn [i account]
                   (-> account
                       (assoc :db/id (nth tempids i)
                              :ledger-account/created-at instant)
                       (update :ledger-account/code (fn [code] (str handle ":" code)))
                       (identify :ledger-account/id)))
                 accounts)))

;;; ----------------------------------------------------------------------------
;;; Seed data

(defn seed-txes
  [seeder]
  (let [instant          (:instant seeder)
        jcf-accounts     (accounts-tx seeder "jcf" ["jcf-acct-0" "jcf-acct-1" "jcf-acct-2"
                                                    "jcf-acct-3" "jcf-acct-4" "jcf-acct-5"
                                                    "jcf-acct-6"])
        charlie-accounts (accounts-tx seeder "charlie" ["charlie-acct-0" "charlie-acct-1" "charlie-acct-2"
                                                        "charlie-acct-3" "charlie-acct-4" "charlie-acct-5"
                                                        "charlie-acct-6"])
        leather-accounts (accounts-tx seeder "leather" ["leather-acct-0" "leather-acct-1" "leather-acct-2"
                                                        "leather-acct-3" "leather-acct-4" "leather-acct-5"
                                                        "leather-acct-6"])
        milly-accounts   (accounts-tx seeder "milly" ["milly-acct-0" "milly-acct-1" "milly-acct-2"
                                                      "milly-acct-3" "milly-acct-4" "milly-acct-5"
                                                      "milly-acct-6"])]
    (into
     [;; JCF
      {:db/id       "jcf-domain"
       :domain/name "jcf.bits.page.localhost"}
      (-> {:db/id           "jcf-post-1"
           :post/created-at instant
           :post/text       "Shipped Monero payment research today. Turns out monero-java has JNI bindings that embed directly in the JVM — no external daemon needed."}
          (identify :post/id))
      (-> {:db/id           "jcf-post-2"
           :post/created-at instant
           :post/text       "The SSE architecture is working beautifully. When I publish a post, every connected subscriber sees it appear instantly."}
          (identify :post/id))
      (-> {:db/id           "jcf-post-3"
           :post/created-at instant
           :post/text       "Deep dive into the Datahike vs Datomic decision and why it matters for self-hosters…"}
          (identify :post/id))
      (product-tx seeder
                  {:db/id               "jcf-product-1"
                   :product/title       "Bits Architecture Guide"
                   :product/description "How Bits uses Datomic, SSE morphing, and Component to deliver real-time creator pages."
                   :product/position    1
                   :sku/code            "JCF-ARCH-DIG"
                   :money/amount        499})
      (product-tx seeder
                  {:db/id               "jcf-product-2"
                   :product/title       "SSE Deep Dive"
                   :product/description "Server-Sent Events from first principles to production — connection management, backpressure, and Brotli compression."
                   :product/position    2
                   :sku/code            "JCF-SSE-DIG"
                   :money/amount        299})
      (-> {:db/id              "jcf-tenant"
           :tenant/created-at  instant
           :tenant/domains     ["jcf-domain"]
           :tenant/products    ["jcf-product-1" "jcf-product-2"]
           :tenant/ledger-accounts (mapv :db/id jcf-accounts)
           :creator/posts      ["jcf-post-1" "jcf-post-2" "jcf-post-3"]
           :creator/handle     "jcf"
           :creator/display-name "James"
           :creator/bio        "Building Bits — censorship-resistant infrastructure for creator sovereignty."
           :creator/links      [{:link/icon  :link.icon/github
                                 :link/label "GitHub"
                                 :link/url   "https://github.com/jcf"}
                                {:link/icon  :link.icon/globe
                                 :link/label "Website"
                                 :link/url   "https://jcf.dev"}]}
          (identify :tenant/id))

      ;; Charlie
      {:db/id       "charlie-domain"
       :domain/name "charlie.bits.page.localhost"}
      (-> {:db/id           "charlie-post-1"
           :post/created-at instant
           :post/text       "Found an excellent stick today. Very good stick. 10/10 would fetch again."}
          (identify :post/id))
      (-> {:db/id           "charlie-post-2"
           :post/created-at instant
           :post/text       "Special ball update: still special, still ball. No further questions at this time."}
          (identify :post/id))
      (-> {:db/id           "charlie-post-3"
           :post/created-at instant
           :post/text       "Conducted extensive research on the couch. Conclusion: very comfortable. Will continue monitoring."}
          (identify :post/id))
      (product-tx seeder
                  {:db/id               "charlie-product-1"
                   :product/title       "Stick Rating Field Guide"
                   :product/description "A comprehensive guide to rating sticks by chewability, throwability, and overall fetch factor."
                   :product/position    1
                   :sku/code            "CHL-STICK-DIG"
                   :money/amount        199})
      (product-tx seeder
                  {:db/id               "charlie-product-2"
                   :product/title       "Ball Assessment Report"
                   :product/description "Detailed analysis of ball specialness. Peer reviewed by other dogs."
                   :product/position    2
                   :sku/code            "CHL-BALL-DIG"
                   :money/amount        249})
      (-> {:db/id              "charlie-tenant"
           :tenant/created-at  instant
           :tenant/domains     ["charlie-domain"]
           :tenant/products    ["charlie-product-1" "charlie-product-2"]
           :tenant/ledger-accounts (mapv :db/id charlie-accounts)
           :creator/posts      ["charlie-post-1" "charlie-post-2" "charlie-post-3"]
           :creator/handle     "charlie"
           :creator/display-name "Charles Montgomery"
           :creator/bio        "Charlie likes treats and special ball."
           :creator/links      [{:link/icon  :link.icon/twitter
                                 :link/label "Twitter"
                                 :link/url   "https://twitter.com/charlie"}]}
          (identify :tenant/id))

      ;; Leather Emporium
      {:db/id       "leather-domain"
       :domain/name "leather.bits.page.localhost"}
      (-> {:db/id           "leather-post-1"
           :post/created-at instant
           :post/text       "New shipment of full-grain Italian leather arrived. The texture on these hides is exceptional."}
          (identify :post/id))
      (-> {:db/id           "leather-post-2"
           :post/created-at instant
           :post/text       "Workshop tip: always condition your leather goods every 6 months. Your wallet will thank you in 20 years."}
          (identify :post/id))
      (-> {:db/id           "leather-post-3"
           :post/created-at instant
           :post/text       "Behind the scenes look at our saddle-stitching process. Each stitch is made by hand using traditional techniques."}
          (identify :post/id))
      (product-tx seeder
                  {:db/id               "leather-product-1"
                   :product/title       "Hand-Stitched Wallet Pattern"
                   :product/description "Full pattern and tutorial for a classic bifold wallet. Includes leather selection guide."
                   :product/position    1
                   :sku/code            "LTH-WALLET-DIG"
                   :money/amount        999})
      (product-tx seeder
                  {:db/id               "leather-product-2"
                   :product/title       "Belt Kit"
                   :product/description "Everything you need to make a belt — pattern, hardware guide, and finishing techniques."
                   :product/position    2
                   :sku/code            "LTH-BELT-DIG"
                   :money/amount        1499})
      (product-tx seeder
                  {:db/id               "leather-product-3"
                   :product/title       "Leather Care Guide"
                   :product/description "How to clean, condition, and protect your leather goods for decades of use."
                   :product/position    3
                   :sku/code            "LTH-CARE-DIG"
                   :money/amount        399})
      (-> {:db/id              "leather-tenant"
           :tenant/created-at  instant
           :tenant/domains     ["leather-domain"]
           :tenant/products    ["leather-product-1" "leather-product-2" "leather-product-3"]
           :tenant/ledger-accounts (mapv :db/id leather-accounts)
           :creator/posts      ["leather-post-1" "leather-post-2" "leather-post-3"]
           :creator/handle     "leather"
           :creator/display-name "The Leather Emporium"
           :creator/bio        "Purveyors of fine leather goods since 1987."
           :creator/links      [{:link/icon  :link.icon/instagram
                                 :link/label "Instagram"
                                 :link/url   "https://instagram.com/leatheremporium"}
                                {:link/icon  :link.icon/globe
                                 :link/label "Website"
                                 :link/url   "https://leatheremporium.example.com"}]}
          (identify :tenant/id))

      ;; Milly
      {:db/id       "milly-domain"
       :domain/name "milly.bits.page.localhost"}
      (-> {:db/id           "milly-post-1"
           :post/created-at instant
           :post/text       "Finally nailed the microfoam today. The secret? Patience and keeping the steam wand at just the right angle. Decaf oat flat white, obviously."}
          (identify :post/id))
      (-> {:db/id           "milly-post-2"
           :post/created-at instant
           :post/text       "Customer asked for a caffeine shot. Had to explain (politely) that we're a strictly decaf establishment. The froth is the star here, not the jitters."}
          (identify :post/id))
      (-> {:db/id           "milly-post-3"
           :post/created-at instant
           :post/text       "New personal best: 47 seconds from portafilter to perfect rosetta. Would have been faster but got distracted by a squirrel outside."}
          (identify :post/id))
      (product-tx seeder
                  {:db/id               "milly-product-1"
                   :product/title       "Latte Art Masterclass"
                   :product/description "From hearts to rosettas — learn to pour like a pro. Decaf required."
                   :product/position    1
                   :sku/code            "MIL-LATTE-DIG"
                   :money/amount        699})
      (product-tx seeder
                  {:db/id               "milly-product-2"
                   :product/title       "Decaf Bean Buyer's Guide"
                   :product/description "How to pick beans that taste great without the jitters. Written by a dog who knows."
                   :product/position    2
                   :sku/code            "MIL-BEANS-DIG"
                   :money/amount        349})
      (-> {:db/id              "milly-tenant"
           :tenant/created-at  instant
           :tenant/domains     ["milly-domain"]
           :tenant/products    ["milly-product-1" "milly-product-2"]
           :tenant/ledger-accounts (mapv :db/id milly-accounts)
           :creator/posts      ["milly-post-1" "milly-post-2" "milly-post-3"]
           :creator/handle     "milly"
           :creator/display-name "Milly"
           :creator/bio        "Aspiring barista. Froth enthusiast. Decaf only — can't have caffeine, what with being a dog."
           :creator/links      [{:link/icon  :link.icon/instagram
                                 :link/label "Instagram"
                                 :link/url   "https://instagram.com/millythebarista"}
                                {:link/icon  :link.icon/youtube
                                 :link/label "YouTube"
                                 :link/url   "https://youtube.com/@millythebarista"}]}
          (identify :tenant/id))]

     (concat jcf-accounts charlie-accounts leather-accounts milly-accounts))))

(comment
  (let [seeder (make-seeder (time/java-date (time/instant "2025-01-01T00:00:00Z")))]
    @(d/transact (datomic/conn (:datomic system)) (seed-txes seeder)))

  (d/q '[:find (pull ?e [*
                         {:creator/posts [*]}
                         {:creator/links [*]}
                         {:tenant/domains [:domain/name]}
                         {:tenant/products [:product/title
                                            {:product/variants [:variant/name
                                                                {:variant/price [:money/amount
                                                                                 {:money/currency [:db/ident]}]}
                                                                {:variant/sku [:sku/code]}]}]}])
         :where [?e :tenant/id]]
       (datomic/db (:datomic system))))
