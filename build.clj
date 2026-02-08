(ns build
  (:require
   [clojure.tools.build.api :as b]))

(def class-dir "target/classes")
(def uber-file "target/bits.jar")

(def basis
  (delay (b/create-basis {:aliases [:linux-x86_64]})))

(defn clean
  [_]
  (b/delete {:path "target"}))

(defn uber
  [_]
  (clean nil)
  (b/copy-dir {:src-dirs   ["resources" "src"]
               :target-dir class-dir})
  (b/compile-clj {:basis     @basis
                  :class-dir class-dir
                  :src-dirs  ["src"]
                  :ns-compile '[bits.main]})
  (b/uber {:basis     @basis
           :class-dir class-dir
           :uber-file uber-file
           :main      'bits.main}))
