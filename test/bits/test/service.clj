(ns bits.test.service
  (:require
   [bits.test.app :as test.app]
   [hato.client :as http]))

;;; --------------------------------------------------------------------------------------------------------------------
;;; Request

(def ^:private http-client
  (http/build-http-client {:connect-timeout 100}))

(defn- cleanup-hato-response
  [response]
  (-> response
      (select-keys #{:body :headers :status})
      (update :headers #(into (sorted-map) (dissoc % ":status")))))

(defn request
  [service request-options]
  (-> (merge {:throw-exceptions? false} request-options)
      (assoc :http-client http-client)
      (update :url #(test.app/service-url service %))
      http/request
      cleanup-hato-response))
