(ns bits.morph
  (:require
   [bits.brotli :as brotli]
   [bits.crypto :as crypto]
   [bits.data :as data]
   [bits.html :as html]
   [bits.spec]
   [bits.string :as string]
   [buddy.core.codecs :as buddy.codecs]
   [buddy.core.hash :as buddy.hash]
   [clojure.core.async :as a]
   [clojure.string :as str]
   [io.pedestal.log :as log]
   [medley.core :as medley]
   [org.httpkit.server :as server]
   [ring.util.response :as response])
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
  [title]
  {:pre [(string? title)]}
  (sse-event "title" (content-hash title) title))

(defn redirect-event
  [url]
  {:pre [(string? url)]}
  (sse-event "redirect" (content-hash url) url))

(defn reload-event
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
;;; Response wrappers

(defn respond
  [content]
  {::respond content})

(defn redirect
  ([url]
   {::redirect url})
  ([url opts]
   (assoc opts ::redirect url)))

;;; ----------------------------------------------------------------------------
;;; Handlers

(defn send-sse!
  "Send compressed SSE data over http-kit channel."
  [stream event]
  (let [{:keys [ba-out br-out ch]} stream
        body                       (brotli/compress-stream ba-out br-out event)
        response                   {:status  200
                                    :headers {"content-type"     "text/event-stream"
                                              "cache-control"    "no-store"
                                              "content-encoding" "br"}
                                    :body    body}]
    (server/send! ch response false)))

(defn page-handler
  "Returns HTML page with view rendered. SSE takes over for updates."
  [layout-fn view-fn]
  (fn [request]
    {:status  200
     :headers {"content-type" "text/html; charset=utf-8"}
     :body    (html/html (layout-fn request (view-fn request)))}))

(defn render-handler
  "SSE stream that re-renders view on refresh signals. Brotli compressed.
   Registers channel in the channels atom for REPL inspection.
   Options:
     :on-close - callback fn called with channel-id when connection closes"
  ([view-fn] (render-handler view-fn {}))
  ([view-fn {:keys [on-close]}]
   (fn [request]
     (let [randomizer   (get-in request [:bits.middleware/state :randomizer])
           channels     (::channels request)
           channel-id   (crypto/random-sid randomizer)
           refresh-mult (::refresh-mult request)
           <refresh     (a/tap refresh-mult (a/chan (a/dropping-buffer 1)))
           <cancel      (a/chan)
           last-id      (response/get-header request "last-event-id")
           sid          (get-in request [:session :sid])
           user-id      (get-in request [:session :user/id])
           request      (assoc request ::channel-id channel-id)]
       (a/>!! <refresh :init)
       (server/as-channel request
                          {:on-open
                           (fn [ch]
                             (log/debug :msg        "Channel opened"
                                        :channel-id channel-id
                                        :sid        sid
                                        :user/id    user-id)
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
                             (log/trace :msg "Channel closed" :channel-id channel-id :sid sid)
                             (swap! channels dissoc channel-id)
                             (when on-close (on-close channel-id))
                             (a/>!! <cancel :stop)
                             (a/untap refresh-mult <refresh))})))))

;;; ----------------------------------------------------------------------------
;;; Action registry
;;;
;;; Actions are normalized once at load time, not per-request.
;;; Specs are in bits.spec to avoid cyclic dependencies.

(defn- normalize-entry
  [entry]
  (if (fn? entry)
    {:handler entry}
    entry))

(defn normalize-actions
  "Normalize action registry at load time. Converts fn → {:handler fn}."
  [actions]
  (into {} (map (fn [[k v]] [k (normalize-entry v)])) actions))

(defn actions->schema
  "Build a Malli multi schema from a normalized actions registry.
   Uses safe whitelist lookup for coercion."
  [actions]
  (let [valid-actions   (data/keyset actions)
        string->keyword (medley/index-by string/keyword->string valid-actions)
        ;; Dispatch runs on raw input before transformation
        dispatch        (fn dispatch-action
                          [m]
                          (or (get string->keyword (:action m))
                              (:action m)))
        ;; Malli requires a fn, not a map (:malli.transform/invalid-transformer)
        decode-action   #(get string->keyword %)
        action-schema   [:fn {:decode/string decode-action} valid-actions]]
    (into [:multi {:dispatch dispatch}]
          (for [[action {:keys [params]}] actions]
            [action (into [:map [:action action-schema]]
                          (or params []))]))))

(defn action-handler
  "Dispatches actions from a normalized registry. Actions return:
   - A Ring response map (with :status) - passed through directly
   - A respond wrapper - returns 200 with rendered HTML
   - Anything else - signals refresh with 204"
  [actions]
  (fn [request]
    (let [refresh-ch (::refresh-ch request)
          action     (get-in request [:parameters :form :action])
          handler    (get-in actions [action :handler])]
      (if handler
        (let [result (handler request)]
          (cond
            (:status result)
            result

            (::redirect result)
            (-> result
                (dissoc ::redirect)
                (assoc :status 200
                       :headers {"location" (::redirect result)}
                       :body ""))

            (::respond result)
            {:status  200
             :headers {"content-type" "text/html; charset=utf-8"}
             :body    (html/htmx (::respond result))}

            :else
            (do
              (a/put! refresh-ch :action)
              {:status 204})))
        (do
          (log/warn :msg "Unknown action" :action action)
          {:status 400
           :body   (str "Unknown action: " action)})))))

;;; ----------------------------------------------------------------------------
;;; Middleware

(defn wrap-refresh
  [handler refresh-ch refresh-mult]
  (fn [request]
    (handler (assoc request
                    ::refresh-ch   refresh-ch
                    ::refresh-mult refresh-mult))))

(defn wrap-channels
  [handler channels]
  (fn [request]
    (handler (assoc request ::channels channels))))

;;; ----------------------------------------------------------------------------
;;; Utilities

(defn throttle
  "Rate-limits channel, emitting at most one value per `ms` milliseconds.
   Input channel should be buffered. Output channel has no buffer."
  [<in ms]
  (let [<out (a/chan)]
    (Thread/startVirtualThread
     (bound-fn []
       (loop []
         (when-some [v (a/<!! <in)]
           (a/>!! <out v)
           (Thread/sleep ^long ms)
           (recur)))
       (a/close! <out)))
    <out))
