(ns bits.next
  (:require
   [bits.html :as html]
   [bits.session :as session]
   [clojure.spec.alpha :as s]
   [com.stuartsierra.component :as component]
   [org.httpkit.server :as server]
   [reitit.ring :as ring]
   [ring.middleware.session :as middleware.session]
   [ring.middleware.session.cookie :as middleware.session.cookie]
   [steffan-westcott.clj-otel.api.trace.span :as span]))

(defn layout
  [request & content]
  [:html {:lang "en"}
   [:head
    [:meta {:name "viewport" :content "width=device-width"}]
    [:title "Bits"]
    [:link {:rel "icon" :href "data:,"}]
    [:script {:src "/idiomorph@0.7.4.min.js"}]]
   (into [:body] content)])

(def routes
  ["/"
   {:get
    (fn [request]
      {:status  200
       :headers {"Content-Type" "text/html; charset=utf-8"}
       :session {:user/id "bits"}
       :body    (html/html
                 (layout request [:main "Please don't forget to tip your server."]))})}])

(defn make-app
  [service]
  (let [{:keys [cookie-name cookie-secret]} service

        ;; Guard against Ring generating a random cookie secret and breaking all
        ;; of our sessions.
        _            (s/assert bytes? cookie-secret)
        cookie-store (middleware.session.cookie/cookie-store {:key cookie-secret})]

    (ring/ring-handler
     (ring/router routes)
     (ring/routes
      (ring/create-resource-handler {:path "/"}))
     {:middleware [[middleware.session/wrap-session {:cookie-attrs {:http-only true
                                                                    :secure    true}
                                                     :cookie-name  cookie-name
                                                     :store        cookie-store}]]})))

(defrecord Service [http-host http-port stop-fn]
  component/Lifecycle
  (start [this]
    (span/with-span! {:name ::start-service}
      (assoc this :stop-fn (server/run-server (make-app this)
                                              {:host                       http-host
                                               :legacy-unsafe-remote-addr? false
                                               :port                       http-port}))))
  (stop [this]
    (span/with-span! {:name ::stop-service}
      (when-let [stop (:stop-fn this)]
        (stop :timeout 200))
      (assoc this :stop-fn nil))))

(defn make-service
  [config]
  (map->Service config))
