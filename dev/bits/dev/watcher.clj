(ns bits.dev.watcher
  (:require
   [camel-snake-kebab.core :as csk]
   [clojure.spec.alpha :as s]
   [clojure.string :as str]
   [com.stuartsierra.component :as component]
   [com.stuartsierra.component.repl :refer [system]]
   [io.pedestal.log :as log])
  (:import
   (java.nio.file FileSystems Paths StandardWatchEventKinds)))

(s/def ::path string?)
(s/def ::handler fn?)
(s/def ::config (s/keys :req-un [::handler ::path]))

(defn- event->map
  [^java.nio.file.WatchEvent event]
  {:kind (csk/->kebab-case-symbol (str (.kind event)))
   :path (str (.context event))})

(defrecord Watcher [handler path thread watch-service]
  component/Lifecycle
  (start [this]
    (let [ws  (.newWatchService (FileSystems/getDefault))
          dir (Paths/get path (into-array String []))]
      (.register dir ws (into-array [StandardWatchEventKinds/ENTRY_CREATE
                                     StandardWatchEventKinds/ENTRY_MODIFY]))
      (assoc this
             :watch-service ws
             :thread        (Thread/startVirtualThread
                             (bound-fn []
                               (try
                                 (loop []
                                   (when-let [k (.take ws)]
                                     (let [events (.pollEvents k)]
                                       (try
                                         (handler this (into #{} (map event->map) events))
                                         (catch Exception ex
                                           (log/warn :msg       "Error calling handler?!"
                                                     :path      path
                                                     :exception ex))))
                                     (.reset k)
                                     (recur)))
                                 (catch java.nio.file.ClosedWatchServiceException _
                                   nil)))))))
  (stop [this]
    (some-> this :watch-service .close)
    (assoc this :watch-service nil :thread nil)))

(defn make-watcher
  [config]
  {:pre [(s/valid? ::config config)]}
  (map->Watcher config))

(comment
  (:watch-service (::watcher system))
  (:thread (::watcher system)))
