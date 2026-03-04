;;; Directory Local Variables         -*- no-byte-compile: t; -*-
((clojure-mode
  . ((cider-clojure-cli-aliases . ":dev:test")
     (cider-ns-refresh-after-fn . "bits.dev/after-refresh")
     (cider-ns-refresh-before-fn . "bits.dev/before-refresh")
     (cider-preferred-build-tool . clojure-cli)
     (cider-redirect-server-output-to-repl . nil)
     (cider-test-default-exclude-selectors . ":e2e :generative")
     (eval . (define-clojure-indent
              (action-button 1)
              (card 1)
              (form 2)
              (page-center 1)
              (page-title 1)
              (text-muted 1))))))
