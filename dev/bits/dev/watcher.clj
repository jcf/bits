(ns bits.dev.watcher
  (:require
   [camel-snake-kebab.core :as csk]
   [clojure.spec.alpha :as s]
   [com.stuartsierra.component :as component]
   [com.stuartsierra.component.repl :refer [system]]
   [io.pedestal.log :as log])
  (:import
   (java.nio.file FileSystems Paths StandardWatchEventKinds)
   (java.util.regex Pattern)))

;;; ----------------------------------------------------------------------------
;;; Specs

(s/def ::path string?)
(s/def ::pattern #(instance? Pattern %))
(s/def ::handler fn?)
(s/def ::watch (s/keys :req-un [::path ::handler]
                       :opt-un [::pattern]))
(s/def ::watches (s/coll-of ::watch :min-count 1))
(s/def ::config (s/keys :req-un [::watches]))

;;; ----------------------------------------------------------------------------
;;; Events

(defn- event->map
  [^java.nio.file.WatchEvent event]
  {:kind (csk/->kebab-case-symbol (str (.kind event)))
   :path (str (.context event))})

(defn- matches-pattern?
  [pattern path]
  (or (nil? pattern)
      (re-matches pattern path)))

(defn- filter-events
  [pattern events]
  (if pattern
    (filter #(matches-pattern? pattern (:path %)) events)
    events))

;;; ----------------------------------------------------------------------------
;;; Component

(defrecord Watcher [watches watch-service key->watch thread]
  component/Lifecycle
  (start [this]
    (let [ws        (.newWatchService (FileSystems/getDefault))
          kinds     (into-array [StandardWatchEventKinds/ENTRY_CREATE
                                 StandardWatchEventKinds/ENTRY_MODIFY])
          key->watch (into {}
                           (for [{:keys [path] :as watch} watches
                                 :let [dir (Paths/get path (into-array String []))
                                       k   (.register dir ws kinds)]]
                             [k watch]))]
      (assoc this
             :watch-service ws
             :key->watch    key->watch
             :thread        (Thread/startVirtualThread
                             (bound-fn []
                               (try
                                 (loop []
                                   (when-let [k (.take ws)]
                                     (let [{:keys [handler pattern path]} (get key->watch k)
                                           events (->> (.pollEvents k)
                                                       (map event->map)
                                                       (filter-events pattern)
                                                       set)]
                                       (when (seq events)
                                         (try
                                           (handler this events)
                                           (catch Exception ex
                                             (log/warn :msg       "Error calling handler?!"
                                                       :path      path
                                                       :exception ex)))))
                                     (.reset k)
                                     (recur)))
                                 (catch java.nio.file.ClosedWatchServiceException _
                                   nil)))))))
  (stop [this]
    (some-> this :watch-service .close)
    (assoc this :watch-service nil :key->watch nil :thread nil)))

(defn make-watcher
  [config]
  {:pre [(s/valid? ::config config)]}
  (map->Watcher config))

(comment
  (:watches (::watcher system))
  (:key->watch (::watcher system)))
