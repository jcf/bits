;;; Directory Local Variables         -*- no-byte-compile: t; -*-
((clojure-mode
  . ((cider-ns-refresh-after-fn . "bits.dev/after-refresh")
     (cider-ns-refresh-before-fn . "bits.dev/before-refresh")
     (cider-clojure-cli-aliases . ":dev:test")
     (cider-redirect-server-output-to-repl . nil)
     (cider-preferred-build-tool . clojure-cli))))
