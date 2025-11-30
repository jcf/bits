(ns bits.csp
  (:require
   [clojure.string :as str]))

;; Extended version of Pedestal's `csp-map->str` with the addition of
;; `:script-src-elem` and `:style-src-attr`.
(defn csp-map->str
  [options]
  (if (string? options)
    options
    (str/join "; "
              (map (fn [[k v]] (str (name k) " " v))
                   (select-keys options [:base-uri
                                         :default-src
                                         :script-src
                                         :script-src-elem
                                         :object-src
                                         :style-src
                                         :style-src-attr
                                         :img-src
                                         :media-src
                                         :frame-src
                                         :child-src
                                         :frame-ancestors
                                         :font-src
                                         :connect-src
                                         :manifest-src
                                         :form-action
                                         :sandbox
                                         :script-nonce
                                         :plugin-types
                                         :reflected-xss
                                         :block-all-mixed-content
                                         :upgrade-insecure-requests
                                         :referrer
                                         :report-uri
                                         :report-to])))))

(defn- qs [s] (format "'%s'" s))

(defn policy
  []
  {:default-src    (qs "self")
   :img-src        (qs "self")
   :object-src     (qs "none")
   :script-src     (qs "self")
   :style-src      (qs "self")
   :style-src-attr (qs "none")})
