(ns bits.next
  (:require
   [bits.html :as html]
   [steffan-westcott.clj-otel.api.trace.span :as span]
   [com.stuartsierra.component :as component]
   [org.httpkit.server :as server]
   [reitit.ring :as ring]))

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
       :body    (html/html
                 (layout request [:main "Please don't forget to tip your server."]))})}])

(def app
  (ring/ring-handler
   (ring/router routes)
   (ring/routes
    (ring/create-resource-handler {:path "/"}))))

(defrecord Service [http-host http-port stop-fn]
  component/Lifecycle
  (start [this]
    (span/with-span! {:name ::start-service}
      (assoc this :stop-fn (server/run-server app {:host                       http-host
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
