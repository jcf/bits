(ns bits.middleware
  (:require
   [bits.csp :as csp]))

;;; ----------------------------------------------------------------------------
;;; State injection

(defn wrap-state
  "Injects service state into request for handlers."
  [handler service]
  (fn [request]
    (handler (assoc request ::state service))))

;;; ----------------------------------------------------------------------------
;;; Accessors

(defn- get-state
  [request k]
  {:post [(some? %)]}
  (get-in request [::state k]))

(defn request->database
  [request]
  (get-state request :datahike))

(defn request->keymaster
  [request]
  (get-state request :keymaster))

(defn request->pool
  [request]
  (get-state request :pool))

;;; ----------------------------------------------------------------------------
;;; Secure headers

(def ^:private secure-headers
  {"content-security-policy"           (csp/csp-map->str (csp/policy))
   "referrer-policy"                   "strict-origin"
   "strict-transport-security"         "max-age=31536000; includeSubdomains"
   "x-content-type-options"            "nosniff"
   "x-download-options"                "noopen"
   "x-frame-options"                   "DENY"
   "x-permitted-cross-domain-policies" "none"
   "x-xss-protection"                  "1; mode=block"})

(defn wrap-secure-headers
  [handler]
  (fn [request]
    (when-let [response (handler request)]
      (update response :headers merge secure-headers))))
