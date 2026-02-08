(ns build
  (:require
   [clojure.tools.build.api :as b]))

(def basis (delay (b/create-basis {:project "deps.edn"})))
(def class-dir "target/classes")
(def uber-file "target/bits.jar")

(defn clean
  [_]
  (b/delete {:path "target"}))

(defn uber
  [_]
  (clean nil)
  (b/copy-dir {:src-dirs   ["src" "resources"]
               :target-dir class-dir})
  (b/compile-clj {:basis     @basis
                  :ns-compile ['bits.main]
                  :class-dir class-dir
                  :jvm-opts  ["-Dclojure.compiler.direct-linking=true"]})
  (b/uber {:basis    @basis
           :class-dir class-dir
           :uber-file uber-file
           :main     'bits.main}))
