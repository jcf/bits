(ns bits.module.creator
  (:require
   [bits.locale :refer [tru]]
   [bits.middleware :as mw]
   [bits.tailwind :as tw]
   [java-time.api :as time]))

;;; ----------------------------------------------------------------------------
;;; Icons

(def ^:private icon-paths
  {:link.icon/github
   "M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"

   :link.icon/linkedin
   [[:path {:d "M16 8a6 6 0 0 1 6 6v7h-4v-7a2 2 0 0 0-2-2 2 2 0 0 0-2 2v7h-4v-7a6 6 0 0 1 6-6z"}]
    [:rect {:x "2" :y "9" :width "4" :height "12"}]
    [:circle {:cx "4" :cy "4" :r "2"}]]

   :link.icon/globe
   [[:circle {:cx "12" :cy "12" :r "10"}]
    [:line {:x1 "2" :y1 "12" :x2 "22" :y2 "12"}]
    [:path {:d "M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"}]]

   :link.icon/instagram
   [[:rect {:x "2" :y "2" :width "20" :height "20" :rx "5" :ry "5"}]
    [:path {:d "M16 11.37A4 4 0 1 1 12.63 8 4 4 0 0 1 16 11.37z"}]
    [:line {:x1 "17.5" :y1 "6.5" :x2 "17.51" :y2 "6.5"}]]

   :link.icon/twitter
   "M23 3a10.9 10.9 0 0 1-3.14 1.53 4.48 4.48 0 0 0-7.86 3v1A10.66 10.66 0 0 1 3 4s-4 9 5 13a11.64 11.64 0 0 1-7 2c9 5 20 0 20-11.5a4.5 4.5 0 0 0-.08-.83A7.72 7.72 0 0 0 23 3z"

   :link.icon/youtube
   [[:path {:d "M22.54 6.42a2.78 2.78 0 0 0-1.94-2C18.88 4 12 4 12 4s-6.88 0-8.6.46a2.78 2.78 0 0 0-1.94 2A29 29 0 0 0 1 11.75a29 29 0 0 0 .46 5.33A2.78 2.78 0 0 0 3.4 19c1.72.46 8.6.46 8.6.46s6.88 0 8.6-.46a2.78 2.78 0 0 0 1.94-2 29 29 0 0 0 .46-5.25 29 29 0 0 0-.46-5.33z"}]
    [:polygon {:points "9.75 15.02 15.5 11.75 9.75 8.48 9.75 15.02"}]]})

(defn icon-svg
  [{:keys [icon class]}]
  (let [paths (get icon-paths icon)
        base  (tw/with-defaults {:viewBox      "0 0 24 24"
                                 :fill         "none"
                                 :stroke       "currentColor"
                                 :stroke-width "2"
                                 :class        class}
                ["w-3.5" "h-3.5" "shrink-0"])]
    (if (string? paths)
      [:svg base [:path {:d paths}]]
      (into [:svg base] paths))))

;;; ----------------------------------------------------------------------------
;;; Icons (internal)

(def ^:private heart-icon
  [:svg {:viewBox "0 0 24 24" :fill "none" :stroke "currentColor" :stroke-width "2"
         :class ["w-4" "h-4"]}
   [:path {:d "M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"}]])

(def ^:private comment-icon
  [:svg {:viewBox "0 0 24 24" :fill "none" :stroke "currentColor" :stroke-width "2"
         :class ["w-4" "h-4"]}
   [:path {:d "M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"}]])

(def ^:private users-icon
  [:svg {:viewBox "0 0 24 24" :fill "none" :stroke "currentColor" :stroke-width "2"
         :class ["w-3.5" "h-3.5"]}
   [:path {:d "M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"}]
   [:circle {:cx "9" :cy "7" :r "4"}]
   [:path {:d "M23 21v-2a4 4 0 0 0-3-3.87"}]
   [:path {:d "M16 3.13a4 4 0 0 1 0 7.75"}]])

(def ^:private calendar-icon
  [:svg {:viewBox "0 0 24 24" :fill "none" :stroke "currentColor" :stroke-width "2"
         :class ["w-3.5" "h-3.5"]}
   [:rect {:x "3" :y "4" :width "18" :height "18" :rx "2" :ry "2"}]
   [:line {:x1 "16" :y1 "2" :x2 "16" :y2 "6"}]
   [:line {:x1 "8" :y1 "2" :x2 "8" :y2 "6"}]
   [:line {:x1 "3" :y1 "10" :x2 "21" :y2 "10"}]])

;;; ----------------------------------------------------------------------------
;;; Avatar

(defn avatar
  [{:keys [creator size] :or {size :large}}]
  (let [avatar-url   (:creator/avatar-url creator)
        display-name (:creator/display-name creator)
        initial      (first display-name)
        base-classes ["rounded-full" "border-3" "border-surface"
                      "bg-surface-hover" "shrink-0" "overflow-hidden"
                      "flex" "items-center" "justify-center"
                      "font-serif" "text-2xl" "text-accent"]
        size-classes (case size
                       :large  ["w-24" "h-24"]
                       :medium ["w-9" "h-9" "text-sm"]
                       :small  ["w-8" "h-8" "text-xs"])
        classes      (tw/merge-classes (into base-classes size-classes))]
    (if avatar-url
      [:img {:src   avatar-url
             :alt   display-name
             :class classes}]
      [:div {:class classes}
       initial])))

;;; ----------------------------------------------------------------------------
;;; Social Links

(defn social-links
  [{:keys [links]}]
  [:div {:class ["flex" "gap-2" "mb-8" "flex-wrap"]}
   (for [{:link/keys [icon label url]} links]
     [:a {:href  url
          :class ["inline-flex" "items-center" "gap-1.5"
                  "px-3" "py-1.5"
                  "bg-surface-raised" "border" "border-border-subtle"
                  "rounded-full"
                  "text-secondary" "text-[0.8125rem]"
                  "no-underline"
                  "transition-colors" "duration-200"
                  "hover:border-border" "hover:text-primary"]
          :key   (str icon)}
      (icon-svg {:icon icon})
      label])])

;;; ----------------------------------------------------------------------------
;;; Subscribe Block

(defn subscribe-block
  [{:keys [price supporter-count post-count]}]
  [:div {:class ["mb-8" "p-6"
                 "bg-surface-raised" "border" "border-border-subtle"
                 "rounded-2xl"]}
   [:div {:class ["flex" "items-baseline" "gap-2" "mb-2"]}
    [:span {:class ["font-serif" "text-[1.75rem]" "text-primary"]} price]
    [:span {:class ["text-sm" "text-muted"]} (tru "/ month")]]
   [:p {:class ["text-[0.8125rem]" "text-muted" "mb-4"]}
    (tru "Support development. Cancel anytime.")]
   [:button {:class ["block" "w-full" "py-3.5"
                     "border-none" "rounded-lg"
                     "bg-accent" "text-surface"
                     "font-sans" "text-[0.9375rem]" "font-semibold"
                     "cursor-pointer"
                     "tracking-wide"
                     "transition-opacity" "duration-150"
                     "hover:opacity-90"]}
    (tru "Subscribe")]
   [:div {:class ["flex" "justify-center" "gap-6" "mt-4"
                  "text-xs" "text-muted"]}
    [:span {:class ["flex" "items-center" "gap-1"]}
     users-icon
     (tru "{0} supporters" supporter-count)]
    [:span {:class ["flex" "items-center" "gap-1"]}
     calendar-icon
     (tru "{0} posts" post-count)]]])

;;; ----------------------------------------------------------------------------
;;; Presence Indicator

(defn presence-indicator
  [{:keys [viewer-count]}]
  [:div {:class ["inline-flex" "items-center" "gap-1.5"
                 "text-xs" "text-muted" "mb-8"]}
   [:span {:class ["w-1.5" "h-1.5" "bg-success" "rounded-full" "animate-pulse"]}]
   (tru "{0} people here now" viewer-count)])

;;; ----------------------------------------------------------------------------
;;; Feed Tabs

(defn feed-tabs
  [{:keys [active-tab]}]
  (let [tabs [{:id :posts :label (tru "Posts")}
              {:id :media :label (tru "Media")}
              {:id :about :label (tru "About")}]]
    [:div {:class ["flex" "gap-0"
                   "border-b" "border-border-subtle"
                   "mb-8"]}
     (for [{:keys [id label]} tabs]
       [:button {:class (tw/merge-classes
                         (into ["px-4" "py-2"
                                "text-sm" "cursor-pointer"
                                "border-b-2" "border-t-0" "border-l-0" "border-r-0"
                                "bg-transparent" "font-sans"
                                "transition-colors" "duration-150"]
                               (if (= id active-tab)
                                 ["text-primary" "border-accent"]
                                 ["text-muted" "border-transparent" "hover:text-secondary"])))
                 :key   (name id)}
        label])]))

;;; ----------------------------------------------------------------------------
;;; Profile Header

(defn profile-header
  [{:keys [creator viewer-count stats]}]
  (let [{:creator/keys [bio display-name handle links]} creator
        {:keys [price supporter-count post-count]}      stats]
    [:div {:class ["w-full" "max-w-[40rem]" "-mt-14" "mx-auto" "px-4"
                   "relative" "z-10"]}
     ;; Avatar row
     [:div {:class ["flex" "items-end" "gap-4" "mb-6"]}
      (avatar {:creator creator})
      [:div {:class "pb-2"}
       [:h1 {:class ["font-serif" "text-2xl" "text-primary" "leading-tight"]}
        display-name]
       [:div {:class ["text-sm" "text-muted" "mt-0.5"]}
        (str "@" handle)]]]

     ;; Bio
     (when bio
       [:p {:class ["text-[0.9375rem]" "text-secondary" "leading-relaxed" "mb-6"]}
        bio])

     ;; Social links
     (when (seq links)
       (social-links {:links links}))

     ;; Subscribe block
     (subscribe-block {:price           price
                       :supporter-count supporter-count
                       :post-count      post-count})

     ;; Presence
     (when (pos? viewer-count)
       (presence-indicator {:viewer-count viewer-count}))

     ;; Feed tabs
     (feed-tabs {:active-tab :posts})]))

;;; ----------------------------------------------------------------------------
;;; Bits Bar

(defn bits-bar
  [{:keys [request]}]
  (let [user (:session/user request)]
    [:nav {:class ["fixed" "top-0" "left-0" "right-0" "z-50"
                   "flex" "items-center" "justify-between"
                   "px-4" "py-2"
                   "bg-surface/85" "backdrop-blur-md"
                   "border-b" "border-border-subtle"]}
     [:a {:href "/" :class ["font-sans" "font-bold" "text-sm" "text-primary"
                            "no-underline" "tracking-wide"]}
      "bits" [:span {:class "text-accent"} "."] "page"]
     [:div {:class ["flex" "gap-2" "items-center"]}
      (if (:user/id user)
        [:a {:href  "/dashboard"
             :class ["px-3.5" "py-1.5"
                     "border" "border-border" "rounded-md"
                     "bg-transparent" "text-secondary" "text-[0.8125rem]"
                     "no-underline"
                     "transition-colors" "duration-150"
                     "hover:border-muted" "hover:text-primary"]}
         (tru "Dashboard")]
        (list
         [:a {:href  "/login"
              :class ["px-3.5" "py-1.5"
                      "border" "border-border" "rounded-md"
                      "bg-transparent" "text-secondary" "text-[0.8125rem]"
                      "no-underline"
                      "transition-colors" "duration-150"
                      "hover:border-muted" "hover:text-primary"]
              :key   "login"}
          (tru "Log in")]
         [:a {:href  "/signup"
              :class ["px-3.5" "py-1.5"
                      "border" "border-accent" "rounded-md"
                      "bg-accent" "text-surface" "text-[0.8125rem]"
                      "no-underline"
                      "transition-colors" "duration-150"
                      "hover:opacity-90"]
              :key   "signup"}
          (tru "Sign up")]))]]))

;;; ----------------------------------------------------------------------------
;;; Banner

(defn banner
  [{:keys [creator]}]
  (let [banner-url (:creator/banner-url creator)]
    [:div {:class ["relative" "w-full" "h-[220px]"
                   "bg-gradient-banner" "overflow-hidden"]}
     (when banner-url
       [:img {:src   banner-url
              :alt   ""
              :class ["w-full" "h-full" "object-cover"]}])
     ;; Gradient fade at bottom
     [:div {:class ["absolute" "inset-0"
                    "bg-gradient-to-b" "from-transparent" "via-transparent" "to-surface"]}]]))

;;; ----------------------------------------------------------------------------
;;; Post Components

(defn- post-actions
  [{:keys [likes comments]}]
  [:div {:class ["flex" "gap-4" "px-4" "pb-4" "pt-2"]}
   [:button {:class ["inline-flex" "items-center" "gap-1"
                     "text-[0.8125rem]" "text-muted"
                     "cursor-pointer" "bg-transparent" "border-none" "font-sans"
                     "transition-colors" "duration-150"
                     "hover:text-secondary"]}
    heart-icon
    (str likes)]
   [:button {:class ["inline-flex" "items-center" "gap-1"
                     "text-[0.8125rem]" "text-muted"
                     "cursor-pointer" "bg-transparent" "border-none" "font-sans"
                     "transition-colors" "duration-150"
                     "hover:text-secondary"]}
    comment-icon
    (str comments)]])

(defn- format-timestamp
  [instant]
  (when instant
    (time/format "d MMM yyyy" (time/local-date-time instant "UTC"))))

(defn- post-header
  [{:keys [creator created-at]}]
  [:div {:class ["flex" "items-center" "gap-2" "p-4"]}
   (avatar {:creator creator :size :medium})
   [:div {:class "flex-1"}
    [:div {:class ["text-sm" "font-medium" "text-primary"]}
     (:creator/display-name creator)]
    [:div {:class ["text-xs" "text-muted"]}
     (format-timestamp created-at)]]])

(defn post-card
  [{:keys [creator post/created-at post/image-url post/text]}]
  (let [;; Fake engagement for now
        likes    11
        comments 4]
    [:article {:class ["bg-surface-raised" "border" "border-border-subtle"
                       "rounded-2xl" "overflow-hidden"
                       "transition-colors" "duration-200"
                       "hover:border-border"]}
     (post-header {:creator creator :created-at created-at})

     [:div {:class "px-4 pb-4"}
      [:p {:class ["text-[0.9375rem]" "text-secondary" "leading-relaxed"]}
       text]]

     (when image-url
       [:div {:class ["w-full" "aspect-video"
                      "bg-surface-hover"
                      "flex" "items-center" "justify-center"
                      "text-muted" "text-[0.8125rem]"]}
        "[ image placeholder ]"])

     (post-actions {:likes likes :comments comments})]))

;;; ----------------------------------------------------------------------------
;;; Feed

(defn feed
  [{:keys [creator posts]}]
  [:div {:class ["max-w-[40rem]" "mx-auto" "px-4" "pb-16"
                 "flex" "flex-col" "gap-8"]}
   (for [post posts]
     (post-card (assoc post :creator creator :key (:post/id post))))])

;;; ----------------------------------------------------------------------------
;;; Footer

(defn page-footer
  [request]
  (let [platform-domain (mw/request->platform-domain request)]
    [:footer {:class ["text-center" "py-12" "px-4" "text-xs" "text-muted"]}
     ;; TODO Extract platform/realm domain
     [:a {:href  (str "https://" platform-domain "/")
          :class ["text-muted" "no-underline" "hover:text-secondary"]}
      "bits.page"]
     " · "
     [:a {:href "#" :class ["text-muted" "no-underline" "hover:text-secondary"]}
      (tru "Terms")]
     " · "
     [:a {:href "#" :class ["text-muted" "no-underline" "hover:text-secondary"]}
      (tru "Privacy")]
     [:div {:class ["mt-2" "text-[0.6875rem]" "opacity-60"]}
      (tru "Self-hostable. Open source. Your data, your rules.")]]))

;;; ----------------------------------------------------------------------------
;;; Sticky CTA (mobile)

(defn sticky-cta
  [{:keys [price]}]
  [:div {:class ["hidden" "max-sm:block"
                 "fixed" "bottom-0" "left-0" "right-0"
                 "p-2" "px-4"
                 "bg-surface-raised" "border-t" "border-border"
                 "z-50" "backdrop-blur-md"]}
   [:button {:class ["block" "w-full" "py-3"
                     "border-none" "rounded-lg"
                     "bg-accent" "text-surface"
                     "font-sans" "text-[0.9375rem]" "font-semibold"
                     "cursor-pointer"]}
    (tru "Subscribe — {0}/month" price)]])

;;; ----------------------------------------------------------------------------
;;; View

(defn creator-profile-view
  [request]
  (let [realm        (:session/realm request)
        creator      (select-keys realm [:creator/avatar-url
                                         :creator/banner-url
                                         :creator/bio
                                         :creator/display-name
                                         :creator/handle
                                         :creator/links])
        posts        (->> (:creator/posts realm)
                          (sort-by :post/created-at #(compare %2 %1)))
        ;; TODO: Real data from database
        viewer-count 3
        stats        {:price           "£10"
                      :supporter-count 23
                      :post-count      (count posts)}]
    (list
     (bits-bar {:request request})
     (banner {:creator creator})
     (profile-header {:creator      creator
                      :viewer-count viewer-count
                      :stats        stats})
     (feed {:creator creator :posts posts})
     (page-footer request)
     (sticky-cta {:price (:price stats)}))))

;;; ----------------------------------------------------------------------------
;;; Module

(def module
  {:name    :bits.module/creator
   :routes  []
   :actions {}})
