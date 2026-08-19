(ns bits.lender
  "Lends cached resources pinned for the extent of a call. There is no
  component here: the owner holds the cache, creates it with `make-cache`,
  and drains it when it shuts down."
  (:require
   [bits.spec]
   [clojure.spec.alpha :as s])
  (:import
   (com.github.benmanes.caffeine.cache Cache Caffeine RemovalListener)
   (java.util.concurrent Executor)))

;;; ----------------------------------------------------------------------------
;;; Entry state
;;;
;;; Each entry is (atom {:k k :pins n :release f :resource r :state
;;; :open|:draining}). Both transitions are single swap!s: a borrower can
;;; only pin an :open entry, and release fires exactly once — at eviction
;;; when idle, or at the final checkin of a draining entry.

(defn- make-entry
  [k resource release]
  (atom {:k k :pins 0 :release release :resource resource :state :open}))

(defn- checkout!
  [entry]
  (:ok? (swap! entry (fn pin [{:keys [state] :as m}]
                       (if (= :open state)
                         (assoc (update m :pins inc) :ok? true)
                         (assoc m :ok? false))))))

(defn- checkin!
  [entry]
  (let [{:keys [k pins release resource state]} (swap! entry update :pins dec)]
    (when (and (= :draining state) (zero? pins))
      (release k resource))))

(defn- drain-entry!
  [entry]
  (let [{:keys [k pins release resource]} (swap! entry assoc :state :draining)]
    (when (zero? pins)
      (release k resource))))

;;; ----------------------------------------------------------------------------
;;; Cache

;; Caffeine's default executor runs removal listeners asynchronously, which
;; would let evict!/drain! return before idle entries are released. Release
;; must be synchronous with the eviction that caused it so an owner's stop
;; can rely on drain! having completed.
(def ^:private same-thread-executor
  (reify Executor
    (execute [_ runnable]
      (.run runnable))))

(defn make-cache
  [config]
  {:pre [(s/valid? ::config config)]}
  (-> (Caffeine/newBuilder)
      (.executor same-thread-executor)
      (.maximumSize (long (:max-size config)))
      (.removalListener
       (reify RemovalListener
         (onRemoval [_ _k entry _cause]
           (some-> entry drain-entry!))))
      (.build)))

;;; ----------------------------------------------------------------------------
;;; Lending

;; Livelock guard, not a tunable: a fresh entry can only lose a checkout race
;; if it is evicted again before the borrower pins it.
(def ^:private ^:const max-lend-attempts 100)

(defn lend
  [cache make release k func]
  (when-not cache
    (throw (ex-info "Cannot lend without a cache." {:k k})))
  (loop [attempt 1]
    (let [entry (.get ^Cache cache k (fn open [k'] (make-entry k' (make k') release)))]
      (if (checkout! entry)
        (try
          (func (:resource @entry))
          (finally
            (checkin! entry)))
        (if (< attempt max-lend-attempts)
          (recur (inc attempt))
          (throw (ex-info "Unable to lend; entries drained faster than borrowed."
                          {:attempts attempt :k k})))))))

(defn evict!
  [cache k]
  (some-> ^Cache cache (.invalidate k)))

(defn drain!
  [cache]
  (some-> ^Cache cache .invalidateAll))
