(ns bits.test.telemetry
  (:require
   [clojure.test]
   [io.pedestal.log :as log]
   [steffan-westcott.clj-otel.api.trace.span :as span]))

(defn install!
  []
  (when-not (::instrumented? (meta clojure.test/test-var))
    (let [original clojure.test/test-var]
      (alter-var-root #'clojure.test/test-var
                      (constantly
                       (with-meta
                         (fn [v]
                           (let [m         (meta v)
                                 test-name (str (:ns m) "/" (:name m))]
                             (span/with-span! {:name       test-name
                                               :attributes {"test.name" test-name}}
                               (original v))))
                         {::instrumented? true
                          ::original      original})))
      (log/debug :msg "Test telemetry enabled."))))
