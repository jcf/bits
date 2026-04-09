(ns bits.request
  (:require
   [clojure.string :as str]
   [ring.util.response :as response])
  (:import
   (com.google.common.net InetAddresses)))

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

(defn local?
  [request]
  (let [d (domain request)]
    (or (= "localhost" d)
        (str/ends-with? d ".localhost")
        (and (InetAddresses/isInetAddress d)
             (.isLoopbackAddress (InetAddresses/forString d))))))
