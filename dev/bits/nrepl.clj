(ns bits.nrepl
  (:require
   [babashka.cli :as cli]
   [babashka.fs :as fs]
   [cider.nrepl.middleware]
   [clansi.core :as ansi]
   [clojure.stacktrace :as stacktrace]
   [clojure.string :as str]
   [io.pedestal.log :as log]
   [nrepl.server :as nrepl]
   [refactor-nrepl.middleware]))

(defn- make-nrepl-handler
  "Please don't refactor production."
  []
  (apply nrepl/default-handler
         (conj cider.nrepl.middleware/cider-middleware
               'refactor-nrepl.middleware/wrap-refactor)))

(defn- nrepl-port-file
  []
  (fs/file (System/getProperty "user.dir") ".nrepl-port"))

(def ^:private args-spec
  {:bind {:alias        :b
          :default      "localhost"
          :default-desc "localhost"
          :desc         "An optional address to bind the nREPL server to."
          :ref          "<bind>"
          :validate     (fn [s] (not (str/blank? s)))}
   :port {:alias        :p
          :coerce       :long
          :default      9999
          :default-desc "9999"
          :desc         "An optional port number to bind the nREPL server to."
          :ref          "<port>"
          :validate     (fn [n] (< 0 n 65535))}
   :help {:alias :h
          :desc  "Describe the CLI and its supported options."}})

(defn- parse-args
  [args]
  (let [{:keys [args opts] :as cli} (cli/parse-args args {:spec args-spec})]
    (assoc cli
           :bind  (-> cli :opts :bind)
           :help? (or (contains? opts :help)
                      (= "help" (first args)))
           :port  (-> cli :opts :port))))

(defn -main
  [& args]
  (let [{:keys [bind help? port]} (parse-args args)]
    (if help?
      (do
        (.println *err* (cli/format-opts {:spec  args-spec
                                          :order [:bind :port :help]}))
        (System/exit 1))
      (let [server (nrepl/start-server :bind bind
                                       :port port
                                       :handler (make-nrepl-handler))]
        (spit (nrepl-port-file) (str port))
        (log/info :msg "nREPL server started!" :bind bind :port port)

        (.addShutdownHook
         (Runtime/getRuntime)
         (Thread.
          ^Runnable
          (fn []
            (try
              (.delete (nrepl-port-file))
              (catch Exception exception
                (log/warn :msg       "Failed to delete nREPL port file?!"
                          :file      (nrepl-port-file)
                          :exception exception)))
            (log/info :msg  "Shutting down nREPL server..."
                      :bind bind
                      :port port)
            (nrepl/stop-server server))))

        ;; Wait forever.
        @(promise)))))

(comment
  (cli/parse-args ["-h"] {:spec args-spec})
  (cli/parse-args ["-p" "1234" "-b" "0.0.0.0"] {:spec args-spec})
  (cli/parse-args ["-p" "1234"] {:spec args-spec})
  (cli/parse-args ["help"] {:spec args-spec})
  (cli/format-opts {:spec args-spec :order [:bind :port :help]}))
