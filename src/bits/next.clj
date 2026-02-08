(ns bits.next
  (:require
   [bits.brotli :as brotli]
   [bits.crypto :as crypto]
   [bits.html :as html]
   [buddy.core.bytes :as buddy.bytes]
   [buddy.core.codecs :as buddy.codecs]
   [buddy.core.hash :as buddy.hash]
   [clojure.core.async :as a]
   [clojure.string :as str]
   [com.stuartsierra.component :as component]
   [io.pedestal.log :as log]
   [org.httpkit.server :as server]
   [reitit.ring :as ring]
   [ring.middleware.cookies :as middleware.cookies]
   [ring.middleware.params :as middleware.params]
   [ring.middleware.session :as middleware.session]
   [steffan-westcott.clj-otel.api.trace.span :as span])
  (:import
   (java.time Instant)
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
  [:html {:class "min-h-screen" :lang "en"}
   [:head
    [:meta {:name "viewport" :content "width=device-width"}]
    [:title "Bits"]
    [:link {:rel "icon" :href "data:,"}]
    [:link {:rel "stylesheet" :href "/app.css"}]
    [:script {:src "/idiomorph@0.7.4.min.js"}]
    [:script {:src "/bits.js"}]]
   [:body {:class "min-h-screen"}
    (into [:main#morph {:class "min-h-screen"}] content)]])

;;; ----------------------------------------------------------------------------
;;; Navigation

(def nav-links
  [["/"         "Counter"]
   ["/cursors"  "Cursors"]
   ["/email"    "Email"]
   ["/redirect" "Redirect"]])

(defn nav-header
  [current-path]
  [:nav {:class "flex gap-4 p-4 bg-neutral-100 dark:bg-neutral-800"}
   (for [[path label] nav-links]
     [:a {:href  path
          :class (str "text-sm font-medium "
                      (if (= path current-path)
                        "text-indigo-600 dark:text-indigo-400"
                        "text-neutral-600 dark:text-neutral-400 hover:text-neutral-900"))}
      label])])

;;; ----------------------------------------------------------------------------
;;; Handlers

(defn send-sse!
  "Send compressed SSE data over http-kit channel."
  [stream event]
  (let [{:keys [ba-out br-out ch]} stream
        body                       (brotli/compress-stream ba-out br-out event)
        response                   {:status  200
                                    :headers {"Content-Type"     "text/event-stream"
                                              "Cache-Control"    "no-store"
                                              "Content-Encoding" "br"}
                                    :body    body}]
    (server/send! ch response false)))

(defn page-handler
  "Returns HTML page with view rendered. SSE takes over for updates."
  [layout-fn view-fn]
  (fn [request]
    {:status  200
     :headers {"Content-Type" "text/html; charset=utf-8"}
     :body    (html/html (layout-fn request (view-fn request)))}))

(defn render-handler
  "SSE stream that re-renders view on refresh signals. Brotli compressed.
   Registers channel in the channels atom for REPL inspection.
   Options:
     :on-close - callback fn called with channel-id when connection closes"
  ([view-fn] (render-handler view-fn {}))
  ([view-fn {:keys [on-close]}]
   (fn [request]
     (let [channels     (::channels request)
           channel-id   (crypto/random-sid)
           refresh-mult (::refresh-mult request)
           <refresh     (a/tap refresh-mult (a/chan (a/dropping-buffer 1)))
           <cancel      (a/chan)
           last-id      (get-in request [:headers "last-event-id"])
           sid          (get-in request [:session :sid])
           user-id      (get-in request [:session :user-id])
           request      (assoc request ::channel-id channel-id)]
       (a/>!! <refresh :init)
       (server/as-channel request
                          {:on-open
                           (fn [ch]
                             (log/debug :msg "Channel opened" :channel-id channel-id :sid sid)
                             (Thread/startVirtualThread
                              (bound-fn []
                                (with-open [ba-out (brotli/byte-array-out-stream)
                                            br-out (brotli/compress-out-stream ba-out :window-size 18)]
                                  (let [stream {:ba-out ba-out
                                                :br-out br-out
                                                :ch     ch}
                                        send!  #(send-sse! stream %)
                                        close! #(server/close ch)]
                                    (swap! channels assoc channel-id
                                           {:close!       close!
                                            :connected-at (Instant/now)
                                            :path         (:uri request)
                                            :remote-addr  (:remote-addr request)
                                            :send!        send!
                                            :sid          sid
                                            :user-id      user-id})
                                    (send! (sse-event "channel" channel-id channel-id))
                                    (try
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
                                                (send! (morph-event html-str)))
                                              hash)
                                            recur))

                                          :priority true))
                                      (finally
                                        (swap! channels dissoc channel-id)))))
                                (server/close ch))))

                           :on-close
                           (fn [_ch _status]
                             (log/debug :msg "Channel closed" :channel-id channel-id :sid sid)
                             (swap! channels dissoc channel-id)
                             (when on-close (on-close channel-id))
                             (a/>!! <cancel :stop)
                             (a/untap refresh-mult <refresh))})))))

(defn respond
  [content]
  {::respond content})

(defn redirect
  [url]
  {::redirect url})

(defn action-handler
  "Dispatches actions by name. Actions can return:
   - A Ring response map (with :status) - passed through directly (e.g. redirects)
   - A respond wrapper - returns 200 with rendered HTML (e.g. validation errors)
   - Anything else - signals refresh with 204"
  [actions]
  (fn [request]
    (let [refresh-ch  (::refresh-ch request)
          action-name (get-in request [:params "action"])
          action-fn   (get actions action-name)]
      (if action-fn
        (let [result (action-fn request)]
          (cond
            (:status result)
            result

            (::redirect result)
            {:status  200
             :headers {"Location" (::redirect result)}
             :body    ""}

            (::respond result)
            {:status  200
             :headers {"Content-Type" "text/html; charset=utf-8"}
             :body    (html/htmx (::respond result))}

            :else
            (do
              (a/put! refresh-ch :action)
              {:status 204})))
        (do
          (log/warn :msg "Unknown action" :action action-name)
          {:status 400
           :body   (str "Unknown action: " action-name)})))))

;;; ----------------------------------------------------------------------------
;;; Demo: Counter

(defonce !state (atom {:count 0}))

(defn email-form
  [{:keys [email error success]}]
  [:div#email-demo {:class "p-6 bg-white dark:bg-neutral-900 rounded-lg shadow max-w-sm"}
   [:h3 {:class "text-lg font-semibold mb-4 dark:text-white"} "Email Validation"]
   [:form {:class "space-y-4 transition-opacity inert:opacity-50 inert:cursor-wait"}
    (when error
      [:p {:class "text-sm text-red-600 dark:text-red-400"} error])
    (when success
      [:p {:class "text-sm text-green-600 dark:text-green-400"} success])
    [:input {:type        "email"
             :name        "email"
             :value       (or email "")
             :placeholder "you@example.com"
             :class       "w-full px-3 py-2 border rounded-md dark:bg-neutral-800 dark:border-neutral-700 dark:text-white"}]
    [:button {:type        "button"
              :data-action "validate-email"
              :class       "w-full px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"}
     "Submit"]]])

(defn redirect-demo
  []
  [:div {:class "p-6 bg-white dark:bg-neutral-900 rounded-lg shadow max-w-sm"}
   [:h3 {:class "text-lg font-semibold mb-4 dark:text-white"} "Redirect Demo"]
   [:button {:type        "button"
             :data-action "redirect-demo"
             :class       "px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-500"}
    "Go to example.com"]])

(defn counter-view
  [_request]
  (list
   (nav-header "/")
   [:div {:class "min-h-screen flex flex-col justify-center items-center space-y-2"}
    [:header
     [:h1 {:class "text-4xl"}
      "Count: "
      [:span {:class "font-bold"} (:count @!state)]]]
    [:section {:class "flex space-x-2"}
     [:button
      {:type        "button"
       :class       "rounded-full bg-indigo-600 p-2 text-white shadow-xs hover:bg-indigo-500 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 dark:bg-indigo-500 dark:shadow-none dark:hover:bg-indigo-400 dark:focus-visible:outline-indigo-500"
       :data-action "inc"}
      [:svg
       {:viewBox     "0 0 20 20"
        :fill        "currentColor"
        :data-slot   "icon"
        :aria-hidden "true"
        :class       "size-5"}
       [:path
        {:d "M10.75 4.75a.75.75 0 0 0-1.5 0v4.5h-4.5a.75.75 0 0 0 0 1.5h4.5v4.5a.75.75 0 0 0 1.5 0v-4.5h4.5a.75.75 0 0 0 0-1.5h-4.5v-4.5Z"}]]]
     [:button
      {:type        "button"
       :class       "rounded-full bg-indigo-600 p-2 text-white shadow-xs hover:bg-indigo-500 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 dark:bg-indigo-500 dark:shadow-none dark:hover:bg-indigo-400 dark:focus-visible:outline-indigo-500"
       :data-action "dec"}
      [:svg
       {:viewBox     "0 0 20 20"
        :fill        "currentColor"
        :data-slot   "icon"
        :aria-hidden "true"
        :class       "size-5"}
       [:path
        {:d "M4.75 9.25a.75.75 0 0 0 0 1.5h10.5a.75.75 0 0 0 0-1.5H4.75Z"}]]]]]))

(defn email-view
  ([_request] (email-view _request {}))
  ([_request form-state]
   (list
    (nav-header "/email")
    [:div {:class "min-h-screen flex flex-col justify-center items-center"}
     (email-form form-state)])))

(defn redirect-view
  [_request]
  (list
   (nav-header "/redirect")
   [:div {:class "min-h-screen flex flex-col justify-center items-center"}
    (redirect-demo)]))

;;; ----------------------------------------------------------------------------
;;; Demo: Cursors

(defonce !cursors (atom {}))

(defn update-cursor!
  [channel-id x y]
  (swap! !cursors assoc channel-id [x y (System/currentTimeMillis)]))

(defn remove-cursor!
  [channel-id]
  (swap! !cursors dissoc channel-id))

(def cursor-colors
  ["bg-red-500"    "bg-blue-500"   "bg-green-500"  "bg-yellow-500"
   "bg-purple-500" "bg-pink-500"   "bg-indigo-500" "bg-teal-500"
   "bg-orange-500" "bg-cyan-500"   "bg-lime-500"   "bg-rose-500"])

(defn cursor-color
  [channel-id]
  (nth cursor-colors (mod (hash channel-id) (count cursor-colors))))

(defn cursor-styles
  [cursors]
  [:style {:id "cursor-positions"}
   (html/raw
    (str/join "\n"
              (for [[channel-id [x y _]] cursors]
                (format ".cursor[data-channel=\"%s\"] { left: %dpx; top: %dpx; }"
                        (subs channel-id 0 6) x y))))])

(defn cursor-label
  [channel-id]
  (let [short-id (subs channel-id 0 6)
        color    (cursor-color channel-id)]
    [:div {:class "cursor" :data-channel short-id}
     [:span {:class (str "px-1.5 py-0.5 text-xs font-mono rounded text-white " color)}
      short-id]]))

(defn cursors-view
  [request]
  (let [cursors @!cursors]
    (list
     (nav-header "/cursors")
     [:div {:id               "cursor-container"
            :class            "relative min-h-screen"
            :data-track-mouse "cursor-move"}

      (cursor-styles cursors)

      (for [[cid _] cursors]
        (cursor-label cid))

      [:div {:class "flex flex-col justify-center items-center min-h-screen"}
       [:h1 {:class "text-4xl font-bold dark:text-white"} "Presence Cursors"]
       [:p {:class "text-neutral-500 mt-4"}
        (str (count cursors) " cursor" (when (not= 1 (count cursors)) "s") " active")]]])))

(def actions
  {"inc"            (fn [_req] (swap! !state update :count inc))
   "dec"            (fn [_req] (swap! !state update :count dec))
   "redirect-demo"  (fn [_req]
                      (redirect "https://example.com"))
   "validate-email" (fn [request]
                      (let [email (get-in request [:params "email"] "")]
                        (cond
                          (str/blank? email)
                          (respond (email-view request {:email email
                                                        :error "Email is required"}))

                          (not (str/includes? email "@"))
                          (respond (email-view request {:email email
                                                        :error "Please enter a valid email address"}))

                          :else
                          (respond (email-view request {:email   email
                                                        :success (str "Welcome, " email "!")})))))
   "cursor-move"    (fn [request]
                      (let [channel-id (get-in request [:params "channel"])
                            x          (parse-long (get-in request [:params "x"] "0"))
                            y          (parse-long (get-in request [:params "y"] "0"))]
                        (when (and channel-id x y (< x 10000) (< y 10000))
                          (update-cursor! channel-id x y))))})

;;; ----------------------------------------------------------------------------
;;; Routes

(def routes
  [["/"
    {:get  (page-handler layout counter-view)
     :post (render-handler counter-view)}]
   ["/cursors"
    {:get  (page-handler layout cursors-view)
     :post (render-handler cursors-view {:on-close remove-cursor!})}]
   ["/email"
    {:get  (page-handler layout email-view)
     :post (render-handler email-view)}]
   ["/redirect"
    {:get  (page-handler layout redirect-view)
     :post (render-handler redirect-view)}]
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

(def ^:private safe-methods
  "HTTP methods that don't require CSRF validation."
  #{:get :head :options})

(defn- sse-request?
  "Returns true if request is for SSE stream (read-only, no CSRF needed)."
  [request]
  (some-> (get-in request [:headers "accept"])
          (str/includes? "text/event-stream")))

(defn- csrf-equals?
  "Constant-time comparison of CSRF tokens to prevent timing attacks."
  [expected actual]
  (and (some? expected)
       (some? actual)
       (buddy.bytes/equals? (.getBytes ^String expected "UTF-8")
                            (.getBytes ^String actual "UTF-8"))))

(defn wrap-csrf
  "Adds CSRF token to request, validates on unsafe methods, sets CSRF cookie.
   Generates sid for anonymous users, ensuring CSRF works from first request.
   Only sets cookie when token changes (new session or rotation)."
  [handler {:keys [cookie-name secret]}]
  (fn [request]
    (let [sid            (or (get-in request [:session :sid])
                             (crypto/random-sid))
          expected       (crypto/csrf-token secret sid)
          actual         (get-in request [:params "csrf"])
          current-cookie (get-in request [:cookies cookie-name :value])
          safe?          (or (contains? safe-methods (:request-method request))
                             (sse-request? request))
          valid?         (or safe? (csrf-equals? expected actual))]
      (if valid?
        (cond-> (-> (handler (assoc request ::csrf expected))
                    (update :session (fnil assoc {}) :sid sid))
          (not= expected current-cookie)
          (assoc-in [:cookies cookie-name] {:value     expected
                                            :http-only false
                                            :path      "/"
                                            :same-site :lax
                                            :secure    true}))
        {:status 403
         :body   "Invalid CSRF token"}))))

(defn wrap-channels
  "Injects channels atom into request."
  [handler channels]
  (fn [request]
    (handler (assoc request ::channels channels))))

;;; ----------------------------------------------------------------------------
;;; Refresh

(defn refresh!
  [service]
  (a/put! (:refresh-ch service) :action))

;;; ----------------------------------------------------------------------------
;;; Stats

(defn stats
  [service]
  (let [channels @(:channels service)]
    {:channels (count channels)
     :sessions (count (into #{} (map :sid) channels))}))

;;; ----------------------------------------------------------------------------
;;; App

(defn make-app
  [service]
  (let [{:keys [channels cookie-name csrf-cookie-name csrf-secret refresh-ch refresh-mult session-store]} service]
    (ring/ring-handler
     (ring/router routes)
     (ring/routes
      (ring/create-resource-handler {:path "/"}))
     {:middleware [[wrap-refresh refresh-ch refresh-mult]
                   [wrap-channels channels]
                   [middleware.params/wrap-params]
                   [middleware.cookies/wrap-cookies]
                   [middleware.session/wrap-session {:cookie-attrs {:http-only true
                                                                    :same-site :lax
                                                                    :secure    true}
                                                     :cookie-name  cookie-name
                                                     :store        session-store}]
                   [wrap-csrf {:cookie-name csrf-cookie-name
                               :secret      csrf-secret}]]})))

(defrecord Service [channels
                    cookie-name
                    csrf-cookie-name
                    csrf-secret
                    http-host
                    http-port
                    refresh-ch
                    refresh-mult
                    server-name
                    session-store
                    stop-fn]
  component/Lifecycle
  (start [this]
    (span/with-span! {:name ::start-service}
      (let [channels     (atom {})
            refresh-ch   (a/chan (a/sliding-buffer 1))
            refresh-mult (a/mult refresh-ch)
            this         (assoc this
                                :channels     channels
                                :refresh-ch   refresh-ch
                                :refresh-mult refresh-mult)]
        (assoc this :stop-fn (server/run-server (make-app this)
                                                {:host                       http-host
                                                 :legacy-unsafe-remote-addr? false
                                                 :port                       http-port
                                                 :server-header              server-name})))))
  (stop [this]
    (span/with-span! {:name ::stop-service}
      (doseq [[_ {:keys [close!]}] @(:channels this)]
        (close!))
      (reset! (:channels this) {})
      (when-let [stop (:stop-fn this)]
        (stop :timeout 200))
      (when-let [ch (:refresh-ch this)]
        (a/close! ch))
      (assoc this :channels nil :refresh-ch nil :refresh-mult nil :stop-fn nil))))

(defn make-service
  [config]
  (map->Service config))
