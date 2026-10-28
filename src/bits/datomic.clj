(ns bits.datomic
  (:require
   [bits.async :as async]
   [bits.cryptex :as cryptex]
   [bits.health :as health]
   [bits.schema :as schema]
   [bits.spec]
   [clojure.spec.alpha :as s]
   [clojure.string :as str]
   [com.stuartsierra.component :as component]
   [datomic.api :as d]
   [io.pedestal.log :as log]
   [lambdaisland.uri :as uri]
   [steffan-westcott.clj-otel.api.trace.span :as span]))

;;; ----------------------------------------------------------------------------
;;; Schema

(defn ensure-schema!
  [conn]
  (span/with-span! {:name ::ensure-schema!}
    @(d/transact conn schema/schema)
    (log/info :msg "Schema installed.")))

;;; ----------------------------------------------------------------------------
;;; URIs
;;;
;;; A Datomic URI is `datomic:` followed by an inner URI whose scheme is the
;;; storage protocol and whose host is the database name.

(def ^:private protocols
  #{"mem" "sql"})

(defn parse-uri
  "Parses a Datomic URI, or returns nil when it isn't one we support."
  [s]
  (when (and (string? s) (str/starts-with? s "datomic:"))
    (let [{:keys [host query scheme]} (uri/uri (subs s (count "datomic:")))]
      (when (and (contains? protocols scheme) (seq host))
        (cond-> {:db-name host :protocol scheme}
          query (assoc :storage-uri query))))))

(defn datomic-uri?
  [x]
  (some? (parse-uri x)))

(defn unparse-uri
  [{:keys [db-name protocol storage-uri]}]
  (str "datomic:" (uri/map->URI {:host db-name :query storage-uri :scheme protocol})))

(defn with-db-name
  [uri db-name]
  (some-> (parse-uri uri) (assoc :db-name db-name) unparse-uri))

;;; ----------------------------------------------------------------------------
;;; Connection

(defn connect
  ([uri]
   (connect uri {}))
  ([uri options]
   {:post [(some? %)]}
   (span/with-span! {:name ::connect}
     (let [{:keys [max-retries retry-delay-ms]
            :or   {max-retries    5
                   retry-delay-ms 1000}} options]
       (loop [n 1]
         (let [result (try
                        (d/create-database uri)
                        (d/connect uri)
                        (catch Exception ex
                          (if (< n max-retries)
                            (do
                              (log/info :msg     "Transactor not available, retrying..."
                                        :attempt n
                                        :max     max-retries)
                              ::retry)
                            (throw (ex-info "Unable to connect to transactor?!"
                                            {:uri uri :attempts n}
                                            ex)))))]
           (if (identical? result ::retry)
             (do
               (Thread/sleep retry-delay-ms)
               (recur (inc n)))
             result)))))))

(defn disconnect
  [conn]
  (span/with-span! {:name ::disconnect}
    (d/release conn)))

;; URIs travel wrapped in cryptexes (see the telemetry and secret handling
;; decision record) and are revealed only at the Datomic API calls. Storage
;; credentials live in the URI's storage part; the redacted form drops it so
;; only `datomic:<protocol>://<db-name>` reaches telemetry.
(defn- redacted-uri
  [uri]
  (-> uri cryptex/reveal parse-uri (dissoc :storage-uri) unparse-uri))

(defn open-conn
  [uri]
  (span/with-span! {:name ::open-conn :attributes {"db_uri" (redacted-uri uri)}}
    (d/connect (cryptex/reveal uri))))

;; `d/release` can block indefinitely — bulk sequential releases have twice
;; wedged a peer (see the isolated-tenant-databases decision record) — so
;; every release is bounded and a stuck one is abandoned on its virtual
;; thread.
(defn release-conn
  [timeout-ms uri conn]
  (span/with-span! {:name ::release-conn :attributes {"db_uri" (redacted-uri uri)}}
    (when (= ::timeout
             (deref (async/vfuture (d/release conn)) timeout-ms ::timeout))
      (log/warn :msg (str "Releasing a connection took more than " timeout-ms
                          "ms; abandoned it on its virtual thread.")
                :db-uri (redacted-uri uri)))))

(defn conn
  [datomic]
  {:post [(some? %)]}
  (:conn datomic))

;;; ----------------------------------------------------------------------------
;;; Database

(defn db
  [datomic]
  (-> datomic conn d/db))

;;; ----------------------------------------------------------------------------
;;; Component

(defrecord Datomic [conn uri]
  component/Lifecycle
  (start [this]
    (span/with-span! {:name ::start-database}
      (let [conn (connect uri)]
        (ensure-schema! conn)
        (assoc this :conn conn))))
  (stop [this]
    (span/with-span! {:name ::stop-database}
      (some-> conn disconnect)
      (assoc this :conn nil)))

  ;; Ready once the transactor confirms the latest basis; a cached database
  ;; value would answer without it. Bounded, because sync waits as long as the
  ;; transactor stays away.
  health/Health
  (alive? [_]
    (some? conn))
  (ready? [_]
    (some? (deref (d/sync conn) 2000 nil))))

(defmethod print-method Datomic
  [_ ^java.io.Writer w]
  (.write w "#<Datomic>"))

(defn make-datomic
  [config]
  {:pre [(s/valid? ::config config)]}
  (map->Datomic config))
