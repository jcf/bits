(ns bits.dev.assets
  (:require
   [bits.app :as app]
   [bits.asset :as asset]
   [com.stuartsierra.component :as component]
   [com.stuartsierra.component.repl :refer [system]]
   [java-time.api :as time]
   [selmer.parser :as template]))

(def fonts
  [{:family "DM Sans"
    :style  "normal"
    :weight "400 700"
    :path   "/DMSans.woff2"}
   {:family "DM Serif Display"
    :style  "normal"
    :weight "400"
    :path   "/DMSerifDisplay.woff2"}
   {:family "JetBrains Mono"
    :style  "normal"
    :weight "400 700"
    :path   "/JetBrainsMono.woff2"}])

(def colors
  [{:name "surface"        :value "#0c0c0e"}
   {:name "surface-raised" :value "#161619"}
   {:name "surface-hover"  :value "#1e1e22"}
   {:name "border"         :value "#2a2a30"}
   {:name "border-subtle"  :value "#1e1e24"}
   {:name "primary"        :value "#e8e6e3"}
   {:name "secondary"      :value "#9a9898"}
   {:name "muted"          :value "#636366"}
   {:name "accent"         :value "#c8a2ff"}
   {:name "accent-dim"     :value "#a07ad8"}
   {:name "success"        :value "#4ade80"}
   ;; Gradient colors (for banner and locked content)
   {:name "banner-start"   :value "#1a1028"}
   {:name "banner-mid"     :value "#0f1729"}
   {:name "locked-start"   :value "#2a1f3d"}
   {:name "locked-mid"     :value "#1a1428"}
   {:name "locked-end"     :value "#141220"}])

(defn theme-context
  [buster]
  {:colors       colors
   :fonts        (mapv (fn [{:keys [path] :as font}]
                         (assoc font :url (asset/asset-path buster path)))
                       fonts)
   :generated-at (time/format "yyyy/MM/dd HH:mm:ss" (time/local-date-time))})

(defn generate-tailwind-css!
  [buster]
  (let [ctx (theme-context buster)
        css (template/render-file "templates/tailwind.css.selmer" ctx)]
    (spit "resources/tailwind.css" css)))

(defn -main
  [& _args]
  (let [system (-> (app/system)
                   (component/subsystem #{:buster})
                   component/start)]
    (try
      (generate-tailwind-css! (:buster system))
      (finally
        (component/stop system)))))

(comment
  (template/cache-off!)
  (generate-tailwind-css! (:buster system)))
