(ns bits.string-test
  (:require
   [bits.string :as sut]
   [clojure.string :as str]
   [clojure.test.check.clojure-test :refer [defspec]]
   [clojure.test.check.generators :as gen]
   [clojure.test.check.properties :as prop]))

;;; ----------------------------------------------------------------------------
;;; Generators

(def gen-lowercase
  (gen/fmap str/join (gen/vector (gen/fmap char (gen/choose 97 122)))))

(def gen-simple-keyword
  (gen/fmap keyword (gen/not-empty gen-lowercase)))

;;; ----------------------------------------------------------------------------
;;; remove-prefix

(defspec remove-prefix-removes-exact-prefix 100
  (prop/for-all [prefix gen-lowercase
                 suffix gen-lowercase]
    (= suffix (sut/remove-prefix (str prefix suffix) prefix))))

;;; ----------------------------------------------------------------------------
;;; keyword->string

(defspec keyword->string-round-trip 100
  (prop/for-all [kw gen-simple-keyword]
    (= kw (keyword (sut/keyword->string kw)))))

(defspec keyword->string-preserves-namespace 100
  (prop/for-all [ns-part (gen/not-empty gen-lowercase)
                 name-part (gen/not-empty gen-lowercase)]
    (let [kw (keyword ns-part name-part)
          s  (sut/keyword->string kw)]
      (and (= ns-part (namespace kw))
           (= name-part (name kw))
           (= s (str ns-part "/" name-part))))))
