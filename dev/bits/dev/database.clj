(ns bits.dev.database
  (:require
   [bits.app :as app]
   [bits.postgres :as postgres]
   [com.stuartsierra.component.repl :refer [start stop system]]
   [datomic.api :as d]))

(defn- local-database-url?
  [db-url]
  (let [{:keys [host path]} (postgres/parse-url db-url)
        dbname              (subs path 1)]
    (and (contains? #{"127.0.0.1" "localhost" "::1"} host)
         (contains? #{"bits_dev" "bits_test"} dbname))))

(defn reset-database!
  []
  (let [database-url (get-in (app/read-config) [:postgres :database-url])
        datomic-uri  (get-in (app/read-config) [:datomic :uri])]
    (when-not (local-database-url? database-url)
      (throw (ex-info "Refusing to delete non-local database."
                      {:database-url database-url})))
    (stop)
    (d/delete-database datomic-uri)
    (start)))

(comment
  (local-database-url?
   "jdbc:postgresql://127.0.0.1:5432/bits_test?user=bits&password=please")

  (local-database-url?
   "jdbc:postgresql://54.32.145.66:5432/bits_test?user=bits&password=please")

  (local-database-url?
   "jdbc:postgresql://127.0.0.1:5432/bits_prod?user=bits&password=please")

  ;; Here be dragons! This function will delete whatever database Datomic is
  ;; configured to use within the running system.
  (reset-database!))
