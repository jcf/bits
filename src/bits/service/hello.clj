(ns bits.service.hello
  (:require
   [bits.assets :as assets]
   [bits.html :as html]
   [bits.interceptor :as i]
   [bits.response]
   [bits.tailwind :as tailwind]
   [charred.api :as json]
   [io.pedestal.http.route :as route]))

(defn get-root
  [request]
  {:status  200
   :headers {"Content-Type" "text/html; charset=utf-8"}
   :body
   (html/html
    (html/layout
     request
     [:section
      [:div
       {:class "mx-auto max-w-7xl px-6 py-24 sm:py-32 lg:flex lg:items-center lg:justify-between lg:px-8"}
       [:h1
        {:class "max-w-2xl text-4xl font-semibold tracking-tight text-neutral-900 sm:text-5xl dark:text-neutral-100"}
        "Welcome to Bits"]
       [:div
        {:class "mt-10 flex items-center gap-x-6 lg:mt-0 lg:shrink-0"}
        [:a
         {:href  (route/url-for :bits.route/get-sign-in :request request)
          :class ["rounded-md"
                  "bg-blue-600"
                  "px-3.5"
                  "py-2.5"
                  "text-sm"
                  "font-semibold"
                  "text-white"
                  "shadow-xs"
                  "hover:bg-blue-500"
                  "focus-visible:outline-2"
                  "focus-visible:outline-offset-2"
                  "focus-visible:outline-blue-600"]}
         "Sign in"]]]]))})

(def routes
  #{["/"
     :get [`get-root]
     :route-name :bits.route/get-root]})
