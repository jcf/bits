(ns build
  (:require
   [clojure.java.io :as io]
   [clojure.spec.alpha :as s]
   [clojure.string :as str]
   [clojure.tools.build.api :as b]
   [grasp.api :as g])
  (:import
   (org.fedorahosted.tennera.jgettext Catalog HeaderFields Message PoWriter)))

(def class-dir "target/classes")
(def uber-file "target/bits.jar")

;;; ----------------------------------------------------------------------------
;;; Clean

(defn clean
  [_]
  (b/delete {:path "target"}))

;;; ----------------------------------------------------------------------------
;;; JAR

(defn uber
  [{:keys [aliases] :or {aliases []}}]
  (clean nil)
  (let [basis (b/create-basis {:project "deps.edn" :aliases aliases})]
    (b/copy-dir {:src-dirs   ["src" "resources"]
                 :target-dir class-dir})
    (b/compile-clj {:basis      basis
                    :ns-compile ['bits.cli]
                    :class-dir  class-dir
                    :jvm-opts   ["-Dclojure.compiler.direct-linking=true"]
                    :report     :stderr})
    (b/uber {:basis     basis
             :class-dir class-dir
             :uber-file uber-file
             :main      'bits.cli})))

;;; ----------------------------------------------------------------------------
;;; Locales

(def ^:private translation-vars
  #{'bits.locale/tru
    'bits.locale/trs})

(s/def ::translate
  (s/and (complement vector?)
         (s/cat :sym (fn [x]
                       (and (symbol? x)
                            (translation-vars (g/resolve-symbol x))))
                :args (s/+ any?))))

(defn- form->message
  [form]
  (let [s (second form)]
    (cond
      (string? s) s
      (and (seq? s) (= 'str (first s)) (every? string? (rest s)))
      (apply str (rest s)))))

(defn- analyze-sources
  [source-paths]
  (let [cwd (str (System/getProperty "user.dir") "/")]
    (for [result (g/grasp source-paths ::translate)
          :let [{:keys [line uri]} (meta result)
                message (form->message result)
                file (-> (str uri)
                         (str/replace #"^file:" "")
                         (str/replace cwd ""))]
          :when message]
      {:file    file
       :line    line
       :message message})))

(defn- group-by-message
  [results]
  (->> results
       (group-by :message)
       (sort-by (comp :file first val))
       (map (fn [[message entries]]
              {:message message
               :files   (mapv #(select-keys % [:file :line]) entries)}))))

(defn- make-header
  ^Message []
  (let [hf  (HeaderFields.)
        now (.format (java.text.SimpleDateFormat. "yyyy-MM-dd HH:mmZ")
                     (java.util.Date.))]
    (doseq [[k v] [[HeaderFields/KEY_ProjectIdVersion "Bits 1.0"]
                   [HeaderFields/KEY_PotCreationDate now]
                   [HeaderFields/KEY_MimeVersion "1.0"]
                   [HeaderFields/KEY_ContentType "text/plain; charset=UTF-8"]
                   [HeaderFields/KEY_ContentTransferEncoding "8bit"]]]
      (.setValue hf k v))
    (let [msg (.unwrap hf)]
      (doseq [comment ["Copyright (C) 2026 Invetica"
                       "SPDX-License-Identifier: AGPL-3.0-or-later"]]
        (.addComment msg comment))
      msg)))

(defn- entry->message
  ^Message [{:keys [message files]}]
  (let [msg (Message.)]
    (.setMsgid msg message)
    (doseq [{:keys [file line]} files]
      (if line
        (.addSourceReference msg file line)
        (.addSourceReference msg file)))
    msg))

(defn- write-pot!
  [entries filename]
  (let [catalog (Catalog. true)]
    (.addMessage catalog (make-header))
    (doseq [entry entries]
      (.addMessage catalog (entry->message entry)))
    (io/make-parents filename)
    (with-open [w (io/writer filename)]
      (.write (PoWriter.) catalog w))))

(defn locales-extract
  [_]
  (let [entries (-> (analyze-sources ["src"])
                    group-by-message)]
    (write-pot! entries "resources/locale/messages.pot")
    (println (format "Extracted %d strings to resources/locale/messages.pot"
                     (count entries)))))
