(ns bits.request
  (:require
   [clojure.string :as str]
   [ring.util.response :as response]))

(defn remote-addr
  "Extract client IP from request.
   Checks X-Forwarded-For header first (takes first IP in chain),
   falls back to :remote-addr."
  [request]
  (or (some-> (response/get-header request "x-forwarded-for")
              (str/split #",")
              first
              str/trim)
      (:remote-addr request)))
