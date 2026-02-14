(ns bits.tailwind-test
  (:require
   [bits.tailwind :as sut]
   [clojure.test :refer [deftest is]]))

(deftest merge-classes
  (is (= "text-lg"
         (sut/merge-classes ["text-sm" "text-lg"]))))
