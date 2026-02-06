(ns bits.next
  (:require
   [bits.brotli :as brotli]
   [bits.html :as html]
   [buddy.core.codecs :as buddy.codecs]
   [buddy.core.hash :as buddy.hash]
   [clojure.core.async :as a]
   [clojure.spec.alpha :as s]
   [clojure.string :as str]
   [com.stuartsierra.component :as component]
   [io.pedestal.log :as log]
   [org.httpkit.server :as server]
   [reitit.ring :as ring]
   [ring.middleware.params :as middleware.params]
   [ring.middleware.session :as middleware.session]
   [ring.middleware.session.cookie :as middleware.session.cookie]
   [steffan-westcott.clj-otel.api.trace.span :as span])
  (:import
   (java.util.concurrent Executors)))

(set-agent-send-executor!
 (Executors/newVirtualThreadPerTaskExecutor))

(set-agent-send-off-executor!
 (Executors/newVirtualThreadPerTaskExecutor))

;;; ----------------------------------------------------------------------------
;;; SSE Formatting

(defn content-hash
  "BLAKE3-256 hash of content (64 hex chars)."
  [s]
  {:pre [(string? s)]}
  (-> (buddy.hash/blake3-256 s)
      (buddy.codecs/bytes->hex)))

(defn sse-event
  "Format an SSE event. Multi-line data gets prefixed."
  [event-type event-id data]
  {:pre [(string? event-type)
         (string? event-id)
         (string? data)]}
  (str "event: " event-type "\n"
       "id: " event-id "\n"
       "data: " (str/replace data "\n" "\ndata: ") "\n\n"))

(defn morph-event
  "Format HTML as a morph SSE event. Event ID is BLAKE3 hash for change detection."
  [html-str]
  {:pre [(string? html-str)]}
  (sse-event "morph" (content-hash html-str) html-str))

(defn title-event
  "Update document.title."
  [title]
  {:pre [(string? title)]}
  (sse-event "title" (content-hash title) title))

(defn redirect-event
  "Navigate to URL."
  [url]
  {:pre [(string? url)]}
  (sse-event "redirect" (content-hash url) url))

(defn reload-event
  "Force full page reload."
  []
  (sse-event "reload" (content-hash "reload") ""))

(defn push-url-event
  "Update URL bar without reload (history.pushState)."
  [url]
  {:pre [(string? url)]}
  (sse-event "push-url" (content-hash url) url))

(defn replace-url-event
  "Replace URL bar without reload (history.replaceState)."
  [url]
  {:pre [(string? url)]}
  (sse-event "replace-url" (content-hash url) url))

;;; ----------------------------------------------------------------------------
;;; Layout

(defn layout
  [request & content]
  [:html {:lang "en"}
   [:head
    [:meta {:name "viewport" :content "width=device-width"}]
    [:title "Bits"]
    [:link {:rel "icon" :href "data:,"}]
    [:script {:src "/idiomorph@0.7.4.min.js"}]
    [:script {:src "/bits.js"}]]
   [:body
    (into [:main#morph] content)]])

;;; ----------------------------------------------------------------------------
;;; Handlers

(defn send-sse!
  "Send compressed SSE data over http-kit channel."
  [ch br-out ba-out event]
  (let [compressed (brotli/compress-stream ba-out br-out event)]
    (server/send! ch {:status  200
                      :headers {"Content-Type"     "text/event-stream"
                                "Cache-Control"    "no-store"
                                "Content-Encoding" "br"}
                      :body    compressed}
                  false)))

(defn page-handler
  "Returns HTML page with view rendered. SSE takes over for updates."
  [layout-fn view-fn]
  (fn [request]
    {:status  200
     :headers {"Content-Type" "text/html; charset=utf-8"}
     :body    (html/html (layout-fn request (view-fn request)))}))

(defn render-handler
  "SSE stream that re-renders view on refresh signals. Brotli compressed."
  [view-fn]
  (fn [request]
    (let [refresh-mult (::refresh-mult request)
          <refresh     (a/tap refresh-mult (a/chan (a/dropping-buffer 1)))
          <cancel      (a/chan)
          last-id      (get-in request [:headers "last-event-id"])]
      (a/>!! <refresh :init)
      (server/as-channel request
                         {:on-open
                          (fn [ch]
                            (log/debug :msg "SSE connection opened")
                            (Thread/startVirtualThread
                             (bound-fn []
                               (with-open [ba-out (brotli/byte-array-out-stream)
                                           br-out (brotli/compress-out-stream ba-out :window-size 18)]
                                 (loop [last-hash last-id]
                                   (a/alt!!
                                     <cancel
                                     (do
                                       (a/close! <refresh)
                                       (a/close! <cancel))

                                     <refresh
                                     ([_]
                                      (some->
                                       (let [html-str (html/htmx (view-fn request))
                                             hash     (content-hash html-str)
                                             changed? (not= hash last-hash)]
                                         (when changed?
                                           (send-sse! ch br-out ba-out (morph-event html-str)))
                                         hash)
                                       recur))

                                     :priority true)))
                               (server/close ch))))

                          :on-close
                          (fn [_ch _status]
                            (log/debug :msg "SSE connection closed")
                            (a/>!! <cancel :stop)
                            (a/untap refresh-mult <refresh))}))))

(defn action-handler
  "Dispatches actions by name, signals refresh. Returns 204."
  [actions]
  (fn [request]
    (let [refresh-ch  (::refresh-ch request)
          action-name (get-in request [:params "action"])
          action-fn   (get actions action-name)]
      (if action-fn
        (do
          (action-fn request)
          (a/put! refresh-ch :action)
          {:status 204})
        (do
          (log/warn :msg "Unknown action" :action action-name)
          {:status 400
           :body   (str "Unknown action: " action-name)})))))

;;; ----------------------------------------------------------------------------
;;; Demo: Counter

(defonce !state (atom {:count 0}))

(defn counter-view
  [_request]
  (list
   [:h1 "Count: " (:count @!state)]
   [:div
    [:button {:data-action "inc"} "+"]
    [:button {:data-action "dec"} "-"]]))

(def actions
  {"inc" (fn [_req] (swap! !state update :count inc))
   "dec" (fn [_req] (swap! !state update :count dec))})

;;; ----------------------------------------------------------------------------
;;; Routes

(def routes
  [["/"
    {:get  (page-handler layout counter-view)
     :post (render-handler counter-view)}]
   ["/action"
    {:post (action-handler actions)}]])

;;; ----------------------------------------------------------------------------
;;; Middleware

(defn wrap-refresh
  "Injects refresh channels into request for handlers."
  [handler refresh-ch refresh-mult]
  (fn [request]
    (handler (assoc request
                    ::refresh-ch   refresh-ch
                    ::refresh-mult refresh-mult))))

;;; ----------------------------------------------------------------------------
;;; App

(defn make-app
  [service]
  (let [{:keys [cookie-name cookie-secret refresh-ch refresh-mult]} service

        ;; Guard against Ring generating a random cookie secret and breaking all
        ;; of our sessions.
        _            (s/assert bytes? cookie-secret)
        cookie-store (middleware.session.cookie/cookie-store {:key cookie-secret})]

    (ring/ring-handler
     (ring/router routes)
     (ring/routes
      (ring/create-resource-handler {:path "/"}))
     {:middleware [[wrap-refresh refresh-ch refresh-mult]
                   [middleware.params/wrap-params]
                   [middleware.session/wrap-session {:cookie-attrs {:http-only true
                                                                    :secure    true}
                                                     :cookie-name  cookie-name
                                                     :store        cookie-store}]]})))

(defrecord Service [http-host
                    http-port
                    refresh-ch
                    refresh-mult
                    stop-fn]
  component/Lifecycle
  (start [this]
    (span/with-span! {:name ::start-service}
      (let [refresh-ch   (a/chan (a/sliding-buffer 1))
            refresh-mult (a/mult refresh-ch)
            this         (assoc this
                                :refresh-ch   refresh-ch
                                :refresh-mult refresh-mult)]
        (assoc this :stop-fn (server/run-server (make-app this)
                                                {:host                       http-host
                                                 :legacy-unsafe-remote-addr? false
                                                 :port                       http-port})))))
  (stop [this]
    (span/with-span! {:name ::stop-service}
      (when-let [stop (:stop-fn this)]
        (stop :timeout 200))
      (when-let [ch (:refresh-ch this)]
        (a/close! ch))
      (assoc this :stop-fn nil :refresh-ch nil :refresh-mult nil))))

(defn make-service
  [config]
  (map->Service config))
