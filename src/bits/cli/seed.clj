(ns bits.cli.seed
  (:require
   [bits.datomic :as datomic]
   [bits.seed :as seed]
   [datomic.api :as d]
   [java-time.api :as time]))

(def spec
  {})

(defn run
  [datomic _ctx]
  (let [seeder (seed/make-seeder (time/java-date))]
    @(d/transact (datomic/conn datomic) (seed/seed-txes seeder))
    (println "Seed data applied.")))

(def command
  {:component :datomic
   :desc      "Apply database seeds"
   :fn        run
   :spec      spec})
