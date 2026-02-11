(ns bits.request
  (:require
   [clojure.string :as str]
   [ring.util.response :as response]))

(defn remote-addr
  [request]
  (or (some-> (response/get-header request "x-forwarded-for")
              (str/split #",")
              first
              str/trim)
      (:remote-addr request)))

(defn domain
  [request]
  (let [host (or (response/get-header request "host")
                 (:server-name request))
        idx  (str/index-of host ":")]
    (cond-> host
      (int? idx) (subs 0 idx))))
