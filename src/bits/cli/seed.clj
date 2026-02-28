(ns bits.cli.seed)

(def spec
  {})

(defn run
  [_datomic _ctx]
  (println "Seeding coming soon!"))

(def command
  {:component :datomic
   :desc      "Apply database seeds"
   :fn        run
   :spec      spec})
