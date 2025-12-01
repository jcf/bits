#!/usr/bin/env bb
;; -*- mode: clojure -*-

(ns analyze
  (:require
   [babashka.fs :as fs]
   [cheshire.core :as json]
   [clojure.edn :as edn]
   [clojure.java.io :as io]
   [clojure.string :as str]))

(defn eprintln
  [& more]
  (binding [*out* *err*]
    (apply println more)))

;;; ----------------------------------------------------------------------------
;;; EDN to JSON conversion

(defn edn-file->map
  [file-path]
  (with-open [r (io/reader file-path)]
    (edn/read (java.io.PushbackReader. r))))

(defn convert-edn-to-json
  "Convert EDN file to JSON, making dates ISO strings"
  [edn-file json-file]
  (let [data      (edn-file->map edn-file)
        json-data (json/generate-string data {:date-format "yyyy-MM-dd'T'HH:mm:ss.SSS'Z'"})]
    (spit json-file json-data)))

(defn convert-all-edn-files
  [scraped-dir output-dir]
  (fs/create-dirs output-dir)
  (doseq [edn-file (fs/glob scraped-dir "*.edn")]
    (let [filename     (fs/file-name edn-file)
          json-file    (str output-dir "/" (str/replace filename #"\.edn$" ".json"))]
      (eprintln "Converting" filename "to JSON...")
      (convert-edn-to-json (str edn-file) json-file))))

;;; ----------------------------------------------------------------------------
;;; Main

(defn -main [& args]
  (let [scraped-dir (or (first args) "./data/scraped")
        output-dir  (or (second args) "./data/json")]
    (eprintln "Converting EDN files from" scraped-dir "to" output-dir)
    (convert-all-edn-files scraped-dir output-dir)
    (eprintln "Conversion complete!")))

(apply -main *command-line-args*)
