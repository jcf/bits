(ns bits.health
  (:require
   [clojure.string :as str]))

;;; ----------------------------------------------------------------------------
;;; Protocol

;; Alive: the component's own state is intact, whatever its dependencies are
;; doing. Ready: it can reach what it depends on right now.
(defprotocol Health
  (alive? [component])
  (ready? [component]))

;;; ----------------------------------------------------------------------------
;;; Report

(def ^:private timeout-ms 2000)

(defn- probe
  [f]
  (let [result (deref (future (try (f) (catch Exception _ false))) timeout-ms ::timeout)]
    (cond
      (= ::timeout result) :timeout
      result               :ok
      :else                :failed)))

(defn report
  "Runs `check` (alive? or ready?) against every component at once."
  [check components]
  (let [probes (mapv (fn [[k component]] [k (future (probe #(check component)))])
                     components)]
    (into (sorted-map) (map (fn [[k p]] [k @p])) probes)))

(defn healthy?
  [report]
  (every? #{:ok} (vals report)))

(defn response
  [report]
  {:status  (if (healthy? report) 200 503)
   :headers {"content-type" "text/plain"}
   :body    (str/join (map (fn [[k v]] (str (name k) ": " (name v) "\n")) report))})
