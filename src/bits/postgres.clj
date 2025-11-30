(ns bits.postgres
  (:require
   [babashka.fs :as fs]
   [babashka.process :as proc]
   [bits.spec]
   [camel-snake-kebab.core :as csk]
   [charred.api :as json]
   [clojure.java.io :as io]
   [clojure.spec.alpha :as s]
   [clojure.string :as str]
   [com.stuartsierra.component :as component]
   [honey.sql :as sql]
   [honey.sql.pg-ops]
   [inflections.core :as infl]
   [io.pedestal.log :as log]
   [medley.core :as medley]
   [next.jdbc :as jdbc]
   [next.jdbc.connection :as jdbc.connection]
   [next.jdbc.date-time]
   [next.jdbc.prepare :as jdbc.prepare]
   [next.jdbc.protocols]
   [next.jdbc.result-set :as jdbc.result-set]
   [next.jdbc.specs]
   [ragtime.next-jdbc]
   [ragtime.repl]
   [ragtime.strategy]
   [steffan-westcott.clj-otel.api.trace.span :as span])
  (:import
   (com.zaxxer.hikari HikariDataSource)
   (java.sql PreparedStatement ResultSet ResultSetMetaData)
   (java.sql ResultSet ResultSetMetaData)))

(set! *warn-on-reflection* true)

(comment
  honey.sql.pg-ops/->
  next.jdbc.date-time/read-as-local
  next.jdbc.specs/jdbc-url-format?
  ragtime.next-jdbc/sql-database)

;;; ------------------------------------------------------------------------------------------------------------------
;;; Specs

(comment bits.spec/retain)

;;; ------------------------------------------------------------------------------------------------------------------
;;; Vector operators

(sql/register-op! :<->)
(sql/register-op! :<#>)

;;; ------------------------------------------------------------------------------------------------------------------
;;; JDBC options

(def default-execute-opts
  "Default options to pass to all executions via `next.jdbc`.

  Ensures column names are converted to unqualified keywords to match any
  aliases used in `SELECT` statements (which is consistent with the behaviour of
  `tech.v3.dataset.sql`."
  {:builder-fn jdbc.result-set/as-unqualified-lower-maps})

(def read-only-connect-opts
  "Options map to pass to `connect!` when loading datasets from a SQL database.

  Auto-commit is disabled as to allow batched inserts. We run a read-only
  connection however to be safe by default. You might be bolder than me."
  {:auto-commit false
   :read-only   true})

;;; ------------------------------------------------------------------------------------------------------------------
;;; Arrays

(extend-protocol jdbc.result-set/ReadableColumn
  java.sql.Array
  (read-column-by-label [^java.sql.Array v _]    (vec (.getArray v)))
  (read-column-by-index [^java.sql.Array v _ _]  (vec (.getArray v))))

;;; ------------------------------------------------------------------------------------------------------------------
;;; Time

(extend-protocol jdbc.result-set/ReadableColumn
  java.sql.Timestamp
  (read-column-by-label [^java.sql.Timestamp v _]     (.toInstant v))
  (read-column-by-index [^java.sql.Timestamp v _2 _3] (.toInstant v)))

(extend-protocol jdbc.result-set/ReadableColumn
  java.sql.Date
  (read-column-by-label [^java.sql.Date v _]     (.toLocalDate v))
  (read-column-by-index [^java.sql.Date v _2 _3] (.toLocalDate v)))

;;; ------------------------------------------------------------------------------------------------------------------
;;; JSON

(defn- <-pgobject
  [^org.postgresql.util.PGobject v]
  (let [type_ (.getType v)
        value (.getValue v)]
    (if (#{"jsonb" "json"} type_)
      (if (= "null" value)
        nil
        (let [json? (try (json/read-json value :key-fn keyword)
                         (catch Exception _ nil))]
          (if-let [json json?]
            (with-meta json {:pgtype type_})
            nil)))
      value)))

(defn- ->pgobject
  [v]
  (let [pgtype (or (:pgtype (meta v)) "jsonb")]
    (doto (org.postgresql.util.PGobject.)
      (.setType pgtype)
      (.setValue (json/write-json-str v)))))

(extend-protocol jdbc.result-set/ReadableColumn
  org.postgresql.util.PGobject
  (read-column-by-label [^org.postgresql.util.PGobject v _]
    (<-pgobject v))
  (read-column-by-index [^org.postgresql.util.PGobject v _2 _3]
    (<-pgobject v)))

(defn- set-object
  [^PreparedStatement stmt i val]
  (.setObject stmt i (->pgobject val)))

(extend-protocol jdbc.prepare/SettableParameter
  clojure.lang.IPersistentMap
  (set-parameter [m ^PreparedStatement s i]
    (set-object s i m))

  clojure.lang.IPersistentVector
  (set-parameter [v ^PreparedStatement s i]
    (set-object s i v)))

;;; ------------------------------------------------------------------------------------------------------------------
;;; Result sets

(defn singularize-segmented
  [s]
  (if-let [n (str/last-index-of s "_")]
    (str (subs s 0 n) "_" (-> s (subs (inc n)) infl/singular))
    (infl/singular s)))

(def enum->namespace
  {})

(defn- qualify
  [table-name table-alias]
  (span/with-span! {:name ::qualify}
    (if (str/starts-with? table-alias "qualify__")
      (let [[_ a] (re-find #"^qualify__(.*?)__slash__" table-alias)]
        (format "bits.postgres.%s" a))
      (let [entity-name (-> table-name singularize-segmented csk/->kebab-case-string)]
        (cond->> entity-name
          (not (str/blank? entity-name))
          (str "bits.postgres."))))))

(def qualify-memo (memoize qualify))

(def labels
  {})

(defn- label
  [x]
  (let [unqualified (cond-> x
                      (str/starts-with? x "qualify__")
                      (str/replace #"^qualify__(?:.*?)__slash__" ""))]
    (or (get labels unqualified)
        (csk/->kebab-case-string unqualified))))

(def label-memo (memoize label))

(defn- get-column-label
  [^ResultSetMetaData rsmeta ^Integer i]
  (try
    (.getColumnLabel rsmeta i)
    (catch java.sql.SQLFeatureNotSupportedException _
      "")))

(defn- get-table-name
  [^ResultSetMetaData rsmeta ^Integer i]
  (try
    (.getTableName rsmeta i)
    (catch java.sql.SQLFeatureNotSupportedException _
      "")))

(defn- resultset-builder
  [^ResultSet result-set _opts]
  (let [rsmeta (.getMetaData result-set)
        cols   (mapv (fn [^Integer i]
                       (let [table-name  (get-table-name rsmeta i)
                             table-label (get-column-label rsmeta i)]
                         (if-let [q (some-> (qualify-memo table-name table-label) not-empty)]
                           (keyword q (-> (.getColumnLabel rsmeta i) label-memo))
                           (keyword (-> (.getColumnLabel rsmeta i) label-memo)))))
                     (range 1 (inc (if rsmeta (.getColumnCount rsmeta) 0))))]
    (jdbc.result-set/->MapResultSetBuilder result-set rsmeta cols)))

(defn- column-reader
  [^ResultSet rs ^ResultSetMetaData rsmeta ^Integer integer]
  (let [i         (int integer)
        object    (.getObject rs i)
        type-name (.getColumnTypeName rsmeta i)
        s         (enum->namespace type-name)]
    (try
      (if (and (some? object) (some? s))
        (keyword (str "bits.postgres." s) object)
        object)
      (catch Exception cause
        (throw (ex-info "Failed to read column?!"
                        {:object object :type-name type-name :s s}
                        cause))))))

(def qualified-builder-fn
  (jdbc.result-set/as-maps-adapter resultset-builder column-reader))

(def defaults
  (assoc jdbc/snake-kebab-opts :builder-fn qualified-builder-fn))

;;; ------------------------------------------------------------------------------------------------------------------
;;; Values

(defn values
  [m]
  (persistent!
   (reduce-kv (fn [agg qk v]
                (let [k (-> qk name keyword)]
                  (assert (not (contains? agg k)))
                  (cond-> agg (some? v) (assoc! k v))))
              (transient {})
              m)))

;;; ------------------------------------------------------------------------------------------------------------------
;;; Polymorphism

(defn ->conn
  [connectable]
  (span/with-span! {:name ::->conn}
    (or (::conn connectable) (:conn connectable) connectable)))

;;; ------------------------------------------------------------------------------------------------------------------
;;; Execute!

(defn execute!
  ([connectable query]
   (execute! connectable query nil))
  ([connectable query options]
   (span/with-span! {:name ::execute!}
     (jdbc/execute! (->conn connectable)
                    (span/with-span! {:name ::format} (sql/format query options))
                    (merge defaults options)))))

(defn execute-one!
  ([connectable query]
   (execute-one! connectable query nil))
  ([connectable query options]
   (span/with-span! {:name ::execute-one!}
     (jdbc/execute-one! (->conn connectable)
                        (span/with-span! {:name ::format} (sql/format query options))
                        (merge defaults options)))))

;;; ------------------------------------------------------------------------------------------------------------------
;;; Enums

(defn enums
  [postgres]
  (->> (execute! postgres {:select [[:t.typname :enum_name]
                                    [:e.enumlabel :enum_value]]
                           :from   [[:pg_type :t]]
                           :join   [[:pg_enum :e]
                                    [:= :t.oid :e.enumtypid]]}
                 jdbc/snake-kebab-opts)
       (group-by :pg-type/enum-name)
       (medley/map-vals #(into (sorted-set) (map :pg-enum/enum-value) %))))

;;; ------------------------------------------------------------------------------------------------------------------
;;; Tables

(def tables-never-to-truncate
  "Tables one must never truncate."
  #{"migrations"})

(defn tables
  [postgres]
  (into (sorted-set)
        (map :pg-tables/tablename)
        (execute! postgres {:select [:tablename]
                            :from   [:pg-catalog.pg-tables]
                            :where (into [:and
                                          [:not= "information_schema" :schemaname]
                                          [:not= "pg_catalog" :schemaname]]
                                         (map #(vector :not= % :tablename))
                                         (sort tables-never-to-truncate))}
                  jdbc/snake-kebab-opts)))

;;; --------------------------------------------------------------------------------------------------------------------
;;; Qualify

(defn ^{:arglists '([ns name])} qualified
  [ns k]
  (keyword (format "qualify__%s__slash__%s" (name ns) (name k))))

(comment
  (qualified :foo :bar))

;;; ----------------------------------------------------------------------------
;;; Connect!

(defn get-datasource
  ([]
   (get-datasource {}))
  ([spec]
   (jdbc/get-datasource (merge {:dbtype "postgres"} spec))))

(defn get-connection
  (^java.sql.Connection [source]
   (get-connection source {}))
  (^java.sql.Connection [source opts]
   (span/with-span! {:name ::get-connection}
     (jdbc/get-connection source opts))))

;;; ----------------------------------------------------------------------------
;;; Migrator

(defn migrate
  [migrator]
  (span/with-span! {:name ::migrate}
    (let [{:keys [database-url]} migrator
          ds                     (get-datasource {:jdbcUrl database-url})
          migrations             (ragtime.next-jdbc/load-resources "migrations")
          ops                    (atom [])
          reporter               (fn [_ op id]
                                   (log/info :msg "Applying migration..." :op op :id id)
                                   (swap! ops conj {:op op :id id}))]
      (try
        (ragtime.repl/migrate
         {:datastore  (ragtime.next-jdbc/sql-database ds {:migrations-table "migrations"})
          :migrations migrations
          :reporter   reporter
          :strategy   ragtime.strategy/apply-new})
        @ops
        (catch Exception exception
          (span/add-exception! exception {:escaping? (:throw-exceptions? migrator)})
          (if (:throw-exceptions? migrator)
            (throw (ex-info "Migration error?!"
                            {:migrations migrations}
                            exception))
            (log/warn :msg             "Migration error!?!!"
                      :migration-names (mapv :id migrations)
                      :exception       exception)))))))

(defrecord Migrator [database-url dump-structure? path throw-exceptions?]
  component/Lifecycle
  (start [this]
    (span/with-span! {:name ::start-migrator}
      (let [migrations (migrate this)
            file       (io/file path)]
        (when (and dump-structure?
                   (or (not (.exists file)) (seq migrations)))
          (log/info :msg        "Dumping structure..."
                    :path       path
                    :migrations migrations)
          (span/with-span! {:name ::dump-structure}
            (proc/check
             (proc/process {:cmd       ["pg_dump"
                                        "--host=127.0.0.1"
                                        "--username=bits"
                                        "bits_dev"
                                        "--schema-only"]
                            :extra-env {"PGPASSWORD" "please"}
                            :out       :write
                            :out-file  (io/file "dev-resources/structure.sql")}))))))
    this)
  (stop [this]
    (span/with-span! {:name ::stop-migrator}
      this)))

(defn make-migrator
  [config]
  {:pre [(s/valid? ::config config)]}
  (map->Migrator config))

;;; ----------------------------------------------------------------------------
;;; Pool

(defrecord Pool [crypto database-url datasource]
  component/Lifecycle
  (start [this]
    (span/with-span! {:name ::start-pool}
      (let [ds (jdbc.connection/->pool HikariDataSource {:jdbcUrl database-url})]
        (span/with-span! {:name ::verify-connection}
          (with-open [conn (get-connection ds)]
            (log/trace :msg        "Connection established! Closing."
                       :datasource ds)
            (.close ^java.sql.Connection conn)))
        (assoc this :datasource ds))))
  (stop [this]
    (span/with-span! {:name ::stop-pool}
      (when-let [ds (:datasource this)]
        (log/trace :msg          "Shutting down connection pool..."
                   :database-url database-url)
        (.close ^com.zaxxer.hikari.HikariDataSource ds))
      (assoc this :datasource nil)))

  next.jdbc.protocols/Connectable
  (get-connection [this opts]
    (jdbc/get-connection (:datasource this) opts)))

(defn make-pool
  [config]
  {:pre [(s/valid? ::config config)]}
  (map->Pool config))

(defn release!
  [^java.sql.Connection conn]
  (.close conn))

;;; ----------------------------------------------------------------------------

(comment
  ;; Rollback most recently applied migration
  (let [migrations (ragtime.next-jdbc/load-resources "migrations")]
    (ragtime.repl/rollback {:datastore  (ragtime.next-jdbc/sql-database
                                         (get-datasource {:dbname "bits_dev"})
                                         {:migrations-table "migrations"})
                            :migrations migrations}))

  ;; DANGER: Rollback all migrations!!
  (let [migrations (ragtime.next-jdbc/load-resources "migrations")]
    (ragtime.repl/rollback {:datastore  (ragtime.next-jdbc/sql-database
                                         (get-datasource {:dbname "lab_dev"})
                                         {:migrations-table "migrations"})
                            :migrations migrations}
                           (count migrations)))

  (proc/check
   (proc/process {:cmd       ["psql"
                              "--host=127.0.0.1"
                              "--username=bits"
                              "bits_dev"]
                  :extra-env {"PGPASSWORD" "please"}
                  :in        (io/file "dev-resources/structure.sql")})))
