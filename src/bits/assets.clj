(ns bits.assets
  (:require
   [babashka.fs :as fs]
   [bits.string :as string]
   [buddy.core.codecs :as codecs]
   [buddy.core.hash :as hash]
   [clojure.java.io :as io]
   [clojure.spec.alpha :as s]
   [clojure.string :as str]
   [com.stuartsierra.component :as component]
   [io.pedestal.log :as log]
   [medley.core :as medley]
   [steffan-westcott.clj-otel.api.trace.span :as span]))

;;; --------------------------------------------------------------------------------------------------------------------
;;; Specs

(s/def ::asset-path string?)
(s/def ::busted string?)
(s/def ::content-type string?)
(s/def ::digest string?)
(s/def ::digest-fn fn?)
(s/def ::formatter (s/and string? #(str/includes? % "%s")))
(s/def ::path #(instance? java.nio.file.Path %))
(s/def ::prefix string?)
(s/def ::resource #(instance? java.net.URL %))
(s/def ::resource-path string?)

;; Technically, this is nilable but we'd not be able to work out the content-type in that case.
(s/def ::ext string?)

(s/def ::parsed
  (s/keys :req [::asset-path
                ::content-type
                ::ext
                ::formatter
                ::path
                ::prefix
                ::resource-path]))

(s/def ::asset
  (s/merge ::parsed
           (s/keys :req [::busted ::digest ::resource])))

(s/def ::asset-path->asset
  (s/map-of ::asset-path ::asset))

(s/def ::buster->asset
  (s/map-of ::buster ::asset))

(s/def ::assets
  (s/coll-of ::asset :kind set?))

(s/def ::stomach
  (s/keys :req [::assets ::asset-path->asset ::buster->asset]))

;;; --------------------------------------------------------------------------------------------------------------------
;;; Content type

;; TODO Make content-type explicit or more intelligent.
(def ^:private ext->content-type
  {"css" "text/css"
   "js"  "text/javascript"
   "txt" "text/plain"})

;;; --------------------------------------------------------------------------------------------------------------------
;;; Digest

(s/fdef parse-path
  :args (s/cat :resource-path ::resource-path)
  :ret  ::parsed)

(defn parse-path
  [resource-path]
  (span/with-span! {:name ::parse-path}
    (let [asset-path   (string/remove-prefix resource-path "public")
          path         (fs/path asset-path)
          [prefix ext] (fs/split-ext path)
          content-type (ext->content-type ext)]
      {::asset-path    asset-path
       ::content-type  content-type
       ::ext           ext
       ::formatter     (str prefix "." "%s" "." ext)
       ::resource-path resource-path
       ::path          path
       ::prefix        prefix})))

(s/fdef stomach
  :args (s/cat :resources ::resources :options (s/keys :opt-un [::digest-fn]))
  :ret  ::stomach)

(defn make-stomach
  [resources]
  (span/with-span! {:name ::make-stomach}
    (let [assets (into #{}
                       (map (fn digest-named-resource
                              [resource-path]
                              (if-let [resource (io/resource resource-path)]
                                (let [{::keys [formatter]
                                       :as    parsed} (parse-path resource-path)
                                      digest          (codecs/bytes->b64-str (hash/blake2b-128 resource) true)
                                      busted          (format formatter digest)
                                      digested        {::busted   busted
                                                       ::digest   digest
                                                       ::resource resource}]
                                  (merge parsed digested))
                                (log/warn :msg "Unable to find resource!?" :resource-path resource-path))))
                       resources)]
      {::assets            assets
       ::asset-path->asset (medley/index-by ::asset-path assets)
       ::busted->asset     (medley/index-by ::busted assets)})))

(s/fdef lookup
  :args (s/cat :buster (s/keys :req-un [::stomach]))
  :ret  (s/nilable ::asset))

(defn lookup
  [buster request]
  (span/with-span! {:name ::lookup}
    (when (identical? :get (:request-method request))
      (get (::busted->asset (:stomach buster)) (:uri request)))))

(s/fdef asset-path
  :args (s/cat :buster (s/keys :req-un [::stomach]) :path ::asset-path)
  :ret  (s/nilable ::busted))

(defn asset-path
  [buster path]
  (span/with-span! {:name ::asset-path}
    (::busted (get (::asset-path->asset (:stomach buster)) path))))

;;; --------------------------------------------------------------------------------------------------------------------
;;; Regurgitate

(defn regurgitate
  [buster]
  (span/with-span! {:name ::regurgitate}
    (assoc buster :stomach (make-stomach (:resources buster)))))

;;; --------------------------------------------------------------------------------------------------------------------
;;; Component

(defrecord Buster [resources stomach]
  component/Lifecycle
  (start [this]
    (span/with-span! {:name ::start-buster}
      (assoc this :stomach (make-stomach resources))))
  (stop [this]
    (span/with-span! {:name ::stop-buster}
      (assoc this :stomach nil))))

(defmethod print-method Buster
  [_ ^java.io.Writer w]
  (.write w "#<Buster %s>"))

(defn make-buster
  [config]
  (map->Buster config))
