(ns bits.identifier-test
  (:require
   [bits.identifier :as identifier]
   [clojure.test :refer [are deftest is]]
   [clojure.test.check.clojure-test :refer [defspec]]
   [clojure.test.check.generators :as gen]
   [clojure.test.check.properties :as prop]))

;;; ----------------------------------------------------------------------------
;;; Encode

(deftest encode
  (are [uuid expected] (= expected (identifier/encode uuid))
    #uuid "00000000-0000-0000-0000-000000000000" "0000000000000000000000000"
    #uuid "ffffffff-ffff-ffff-ffff-ffffffffffff" "f5lxx1zz5pnorynqglhzmsp33"
    #uuid "3867b6f3-dbb0-4ef5-8078-364897154fd9" "3c7rc6rbqke4pmmp74vsxvv15"))

;;; ----------------------------------------------------------------------------
;;; Decode

(deftest decode
  (are [in out] (= out (identifier/parse in))
    ""                                     nil
    "abc"                                  nil
    "not-a-uuid-at-all"                    nil
    "3c7rc6rbqke4pmmp74vsxvv15"            #uuid "3867b6f3-dbb0-4ef5-8078-364897154fd9"
    "3C7RC6RBQKE4PMMP74VSXVV15"            #uuid "3867b6f3-dbb0-4ef5-8078-364897154fd9"
    "3867b6f3-dbb0-4ef5-8078-364897154fd9" #uuid "3867b6f3-dbb0-4ef5-8078-364897154fd9"))

(defspec roundtrip
  (prop/for-all [uuid gen/uuid]
    (= uuid (-> uuid identifier/encode identifier/decode))))

(defspec pattern
  (prop/for-all [uuid gen/uuid]
    (re-matches #"[0-9a-z]{25}" (identifier/encode uuid))))
