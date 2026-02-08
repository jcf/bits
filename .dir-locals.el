;;; Directory Local Variables         -*- no-byte-compile: t; -*-
((clojure-mode
  . ((eval . (define-clojure-indent
               (action-button 1)
               (form 2)))
     (cider-ns-refresh-after-fn . "bits.dev/after-refresh")
     (cider-ns-refresh-before-fn . "bits.dev/before-refresh")
     (cider-clojure-cli-aliases . ":dev:test")
     (cider-redirect-server-output-to-repl . nil)
     (cider-preferred-build-tool . clojure-cli))))
