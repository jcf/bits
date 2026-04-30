(ns bits.dev.realm
  (:require
   [bits.datomic :as datomic]
   [bits.seed :as seed]
   [com.stuartsierra.component.repl :refer [system]]
   [datomic.api :as d]
   [java-time.api :as time]))

;;; ----------------------------------------------------------------------------
;;; Delegated API

(def make-seeder seed/make-seeder)
(def seed-txes seed/seed-txes)

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
