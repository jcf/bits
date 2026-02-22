(ns bits.dev.compile
  (:require
   [babashka.fs :as fs]))

(defn -main
  [& _args]
  (let [compile-path "target/classes"]
    (-> compile-path fs/path fs/create-dirs)
    (binding [*compile-files* true
              *compile-path* compile-path]
      (require 'bits.main :reload-all))))
