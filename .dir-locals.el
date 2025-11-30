;;; Directory Local Variables         -*- no-byte-compile: t; -*-
((clojure-mode
  . ((cider-ns-refresh-before-fn . "com.stuartsierra.component.repl/stop")
     (cider-ns-refresh-after-fn . "com.stuartsierra.component.repl/start")
     (cider-clojure-cli-aliases . ":dev:test")
     (cider-redirect-server-output-to-repl . nil)
     (cider-preferred-build-tool . clojure-cli))))
