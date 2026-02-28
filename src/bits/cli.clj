(ns bits.cli
  (:require
   [babashka.cli :as cli]
   [bits.app :as app]
   [bits.cli.seed :as cli.seed]
   [bits.cli.serve :as cli.serve]
   [bits.cli.warmup :as cli.warmup]
   [bits.data :refer [keyset]]
   [clansi.core :as ansi]
   [clojure.spec.alpha :as s]
   [com.stuartsierra.component :as component])
  (:gen-class))

;;; ----------------------------------------------------------------------------
;;; Sysexits

(def ^:const ^:private exit-code
  "BSD sysexits.h standard exit codes."
  {:bits.cli.exit/ok                0
   :bits.cli.exit/base              64
   :bits.cli.exit/usage             64
   :bits.cli.exit/data-error        65
   :bits.cli.exit/no-input          66
   :bits.cli.exit/no-user           67
   :bits.cli.exit/no-host           68
   :bits.cli.exit/unavailable       69
   :bits.cli.exit/software-error    70
   :bits.cli.exit/os-error          71
   :bits.cli.exit/os-file-missing   72
   :bits.cli.exit/cannot-create     73
   :bits.cli.exit/io-error          74
   :bits.cli.exit/temp-failure      75
   :bits.cli.exit/protocol-error    76
   :bits.cli.exit/permission-denied 77
   :bits.cli.exit/config-error      78})

;;; ----------------------------------------------------------------------------
;;; Specs

(s/def ::err  (s/coll-of string? :kind vector?))
(s/def ::out  (s/coll-of string? :kind vector?))
(s/def ::exit (keyset exit-code))

(s/def ::ret
  (s/keys :opt-un [::err ::exit ::out]))

;;; ----------------------------------------------------------------------------
;;; Commands

(def ^:private commands
  {"seed"   cli.seed/command
   "serve"  cli.serve/command
   "warmup" cli.warmup/command})

;;; ----------------------------------------------------------------------------
;;; UI

(defn eprintln [& args] (binding [*out* *err*] (apply println args)))

(defn- header [s] (ansi/style s :bright))

(defn- format-commands
  [cmds]
  (let [rows (mapv (fn [[name {:keys [desc]}]] [name desc]) (sort cmds))]
    (cli/format-table {:rows rows :indent 2})))

(defn usage
  []
  (str (header "Usage:") " bits <command> [options]\n\n"
       (header "Commands:") "\n" (format-commands commands) "\n\n"
       "Run 'bits <command> --help' for command-specific help."))

;;; ----------------------------------------------------------------------------
;;; Tabulate

(defn- command-help
  [{:keys [cmds desc spec]}]
  (let [cmd-name (first cmds)
        usage    (str (header "Usage:") " bits " cmd-name
                      (when (seq spec) " [options]"))]
    (if (seq spec)
      (str usage "\n\n" desc "\n\n" (header "Options:") "\n" (cli/format-opts {:spec spec}))
      (str usage "\n\n" desc))))

(defn- wrap-system
  [component-key run]
  (fn [ctx]
    (let [system    (app/system)
          subsystem (component/subsystem system #{component-key})
          running   (component/start subsystem)
          component (get running component-key)]
      (try
        (run component ctx)
        (finally
          (component/stop running))))))

(defn- wrap-help
  [cmd run]
  (fn [ctx]
    (if (get-in ctx [:opts :help])
      (println (command-help cmd))
      (run ctx))))

(defn- prepare-command
  [cmd]
  (let [{component-key :component
         run           :fn} cmd
        run (if (some? component-key)
              (wrap-system component-key run)
              (fn [ctx] (run nil ctx)))]
    (assoc cmd :fn (wrap-help cmd run))))

(defn- tabulate
  [string->command]
  (into [{:cmds [] :fn (fn [ctx]
                         (if (get-in ctx [:opts :help])
                           (println (usage))
                           ((:fn cli.serve/command) nil ctx)))}]
        (map (fn [[string command]]
               (-> command
                   (assoc :cmds [string])
                   prepare-command)))
        string->command))

;;; ----------------------------------------------------------------------------
;;; CLI

(defn result->exit
  [result]
  (get exit-code (:bits.cli.exit/code result :bits.cli.exit/ok) 0))

(defn run
  [table args]
  (cli/dispatch table args
                {:error-fn (fn [{:keys [cause wrong-input]}]
                             (case cause
                               :no-match
                               (do (eprintln "Unknown command:" wrong-input)
                                   {:bits.cli.exit/code :bits.cli.exit/usage})
                               :input-exhausted
                               (eprintln (usage))))}))

(defn- run-cli
  [args]
  (let [result (run (tabulate commands) args)]
    (System/exit (result->exit result))))

(defn -main
  [& args]
  (let [no-color? (some #{"--no-color"} args)
        args      (remove #{"--no-color"} args)]
    (if no-color?
      (ansi/without-ansi (run-cli args))
      (run-cli args))))
