(ns scrape
  (:require
   [babashka.fs :as fs]
   [babashka.process :as proc]
   [clojure.edn :as edn]
   [clojure.java.io :as io]
   [clojure.pprint :as pprint]
   [clojure.string :as str]
   [etaoin.api :as e]))

(defn eprintln
  [& more]
  (binding [*out* *err*]
    (apply println more)))

;;; ----------------------------------------------------------------------------
;;; Configuration

(def config-file "config.edn")
(def profiles-file "profiles.edn")

(def firefox-profile-path
  (str (System/getProperty "user.home")
       "/Library/Application Support/Firefox/Profiles/konxh2fr.dev-edition-default"))

(defn load-config []
  (-> config-file slurp edn/read-string))


;;; ----------------------------------------------------------------------------
;;; Cookie Management

(defn extract-instagram-cookies
  "Extract Instagram cookies from Firefox cookies.sqlite"
  [profile-path]
  (let [cookies-db (str profile-path "/cookies.sqlite")
        temp-db    "/tmp/firefox-cookies-copy.sqlite"]
    (proc/shell "cp" cookies-db temp-db)
    (try
      (let [result (proc/shell
                    {:out :string}
                    "sqlite3" temp-db
                    "SELECT name, value, host, path, expiry, isSecure, sameSite FROM moz_cookies WHERE host LIKE '%instagram%'")
            lines  (str/split-lines (:out result))]
        (keep (fn [line]
                (when-not (str/blank? line)
                  (let [[name value host path expiry secure same-site] (str/split line #"\|")]
                    {:name      name
                     :value     value
                     :domain    host
                     :path      (or path "/")
                     :expiry    (when expiry (Long/parseLong expiry))
                     :secure    (= secure "1")
                     :http-only false
                     :same-site (case same-site
                                  "0" "None"
                                  "1" "Lax"
                                  "2" "Strict"
                                  nil)})))
              lines))
      (finally
        (fs/delete-if-exists temp-db)))))

(defn load-cookies
  "Load cookies into the browser session"
  [driver cookies]
  (eprintln "Loading" (count cookies) "Instagram cookies...")
  (doseq [cookie cookies]
    (try
      (e/set-cookie driver cookie)
      (catch Exception e
        (eprintln "Warning: Could not set cookie" (:name cookie))))))

;;; ----------------------------------------------------------------------------
;;; URL Utilities

(defn unwrap-instagram-url
  "Unwrap Instagram's l.instagram.com tracking URLs to get the actual destination"
  [url]
  (if (some-> url (str/starts-with? "https://l.instagram.com"))
    (try
      (let [query-string (second (str/split url #"\?"))
            params       (into {}
                               (map (fn [pair]
                                      (let [[k v] (str/split pair #"=" 2)]
                                        [k v]))
                                    (str/split query-string #"&")))]
        (if-let [encoded-url (get params "u")]
          (java.net.URLDecoder/decode encoded-url "UTF-8")
          url))
      (catch Exception _
        url))
    url))

(defn parse-utm-params
  [url]
  (when-let [query-string (second (str/split url #"\?"))]
    (let [params   (into {}
                         (map (fn [pair]
                                (let [[k v] (str/split pair #"=")]
                                  [(keyword k) v]))
                              (str/split query-string #"&")))
          utm-keys [:utm_source :utm_medium :utm_campaign :utm_term :utm_content]]
      (when (some params utm-keys)
        (select-keys params utm-keys)))))

(defn extract-username
  "Extract username from Instagram URL"
  [url]
  (or (second (re-find #"instagram\.com/([^/?]+)" url))
      url))

(defn url-slug
  "Generate a filename-safe slug from a URL"
  [url]
  (try
    (let [host (-> (java.net.URI. url)
                   .getHost
                   (or "unknown"))]
      (-> host
          (str/replace #"^www\." "")
          (str/replace #"\." "-")))
    (catch Exception _
      "unknown")))

(defn identify-platform
  "Identify link-in-bio platform from URL"
  [url]
  (cond
    (str/includes? url "linktr.ee")  :linktree
    (str/includes? url "link.me")    :linkme
    (str/includes? url "beacons.ai") :beacons
    (str/includes? url "hoo.be")     :hoobe
    (str/includes? url "bio.site")   :biosite
    (str/includes? url "solo.to")    :solo
    :else                            (keyword (url-slug url))))

;;; ----------------------------------------------------------------------------
;;; Screenshot Utilities

(defn screenshot-path
  [data-dir username suffix]
  (let [screenshots-dir (str data-dir "/screenshots")]
    (fs/create-dirs screenshots-dir)
    (str screenshots-dir "/" username "-" suffix ".png")))

(defn full-page-screenshot
  "Take a full-page screenshot by resizing viewport to document height"
  [driver path]
  (let [;; Get full document dimensions
        width  (e/js-execute driver "return document.documentElement.scrollWidth;")
        height (e/js-execute driver "return Math.max(document.body.scrollHeight, document.documentElement.scrollHeight, document.body.offsetHeight, document.documentElement.offsetHeight, document.body.clientHeight, document.documentElement.clientHeight);")
        ;; Store current window size
        current-size (e/get-window-size driver)]

    (try
      ;; Resize to full page height (capped at 20000px to avoid issues)
      (e/set-window-size driver width (min height 20000))
      (e/wait 0.5)  ;; Brief wait for resize to settle

      ;; Scroll to top and take screenshot
      (e/js-execute driver "window.scrollTo(0, 0);")
      (e/screenshot driver path)

      (finally
        ;; Restore original window size
        (e/set-window-size driver (:width current-size) (:height current-size))))))

;;; ----------------------------------------------------------------------------
;;; Scraping Functions

(defn should-skip-destination?
  "Skip obvious noise links that don't represent creator destinations"
  [url text]
  (or
   ;; Platform self-promotion
   (str/includes? url "linktr.ee/register")
   (str/includes? url "linktr.ee/discover")
   (str/includes? url "linktr.ee/privacy")
   (str/includes? url "linktr.ee/s/about")
   (str/includes? text "Sign up free")
   (str/includes? text "Learn more about Linktree")
   (str/includes? text "Explore more Linktrees")
   (str/includes? text "Join ")
   (str/includes? text "Privacy")
   (str/includes? text "Report")
   ;; Legal/terms pages
   (str/includes? url "/terms")
   (str/includes? url "/privacy")
   (str/includes? url "/legal")
   (str/includes? url "help.instagram.com")))

(defn scrape-destination
  "Follow an outbound link and capture the final destination page"
  [driver config username url text]
  (when-not (should-skip-destination? url text)
    (try
      (eprintln "  Following destination:" (subs text 0 (min 50 (count text))) "...")
      (e/go driver url)
      (e/wait 3)  ;; Let page load and any redirects settle

      (let [final-url       (e/get-url driver)
            domain          (url-slug final-url)
            text-slug       (let [clean-text (-> text
                                                 (str/replace #"[^a-zA-Z0-9\s]" "")
                                                 (str/replace #"\s+" "-")
                                                 (str/lower-case))]
                              (subs clean-text 0 (min 30 (count clean-text))))
            screenshot-file (screenshot-path (:data-dir config)
                                            username
                                            (str "dest-" domain "-" text-slug))]
        (full-page-screenshot driver screenshot-file)

        {:final-url   final-url
         :screenshot  screenshot-file
         :redirected? (not= url final-url)})
      (catch Exception e
        (eprintln "  Error scraping destination" url ":" (.getMessage e))
        nil))))

(defn dismiss-cookie-banner
  "Wait for and dismiss Instagram's cookie consent banner"
  [driver]
  (try
    (e/wait-visible driver {:xpath "//button[contains(text(), 'Allow all cookies')]"} {:timeout 5})
    (e/click driver {:xpath "//button[contains(text(), 'Allow all cookies')]"})
    (catch Exception _)))

(defn scrape-instagram-profile
  [driver cookies config url]
  (let [username (extract-username url)]
    (try
      ;; Visit Instagram homepage first to set cookies
      (e/go driver "https://www.instagram.com/")
      (e/wait 1)
      (load-cookies driver cookies)

      ;; Now visit the actual profile
      (e/go driver url)
      (e/wait 2)

      (dismiss-cookie-banner driver)

      ;; Scroll to top and wait for content
      (e/js-execute driver "window.scrollTo(0, 0);")
      (e/wait 1)

      (let [bio-text        (try
                              (e/js-execute driver
                                            "return document.querySelector('header') ? document.querySelector('header').innerText : '';")
                              (catch Exception _ ""))
            bio-links       (try
                              (let [all-links (e/js-execute driver
                                                            "return Array.from(document.querySelectorAll('header a[href]')).map(a => a.href);")]
                                (into #{}
                                      (comp
                                       (filter (fn [href]
                                                 (and href
                                                      (not (str/includes? href "instagram.com/accounts/"))
                                                      (not (str/includes? href "instagram.com/explore/"))
                                                      (not (str/includes? href "instagram.com/direct/"))
                                                      (not (str/starts-with? href "javascript:"))
                                                      (or (not (str/includes? href "instagram.com"))
                                                          (re-find #"instagram\.com/[^/]+/?$" href)))))
                                       (map unwrap-instagram-url))
                                      all-links))
                              (catch Exception _ #{}))
            screenshot-file (screenshot-path (:data-dir config) username "ig")]
        (full-page-screenshot driver screenshot-file)

        {:instagram/username   username
         :instagram/url        url
         :instagram/bio        bio-text
         :instagram/bio-links  bio-links
         :instagram/screenshot screenshot-file})
      (catch Exception e
        (eprintln "Error scraping Instagram profile" url ":" (.getMessage e))
        nil))))

(defn try-selector
  "Try to find and click an element with a selector, return true if successful"
  [driver selector timeout]
  (try
    (e/wait-visible driver selector {:timeout timeout})
    (e/click driver selector)
    true
    (catch Exception _
      false)))

(defn dismiss-link-platform-banners
  "Dismiss cookie banners on link-in-bio platforms"
  [driver platform]
  (case platform
    :linktree
    (do
      (eprintln "Attempting to dismiss Linktree banner...")
      ;; Check for iframe containing the banner
      (try
        (eprintln "Looking for iframe...")
        (when-let [iframe (try
                            (e/query driver {:css "iframe[title*='privacy' i], iframe[title*='cookie' i]"})
                            (catch Exception _
                              nil))]
          (eprintln "Found iframe, switching context...")
          (e/switch-frame driver iframe))
        (catch Exception e
          (eprintln "No iframe found or switch failed:" (.getMessage e))))

      (let [selectors [{:css "button[data-testid='CookieBannerAcceptAll']"}
                       {:xpath "//button[text()='Accept All']"}
                       {:xpath "//button[contains(text(), 'Accept All')]"}]
            success   (some #(when (try-selector driver % 2)
                               (eprintln "Banner button found and clicked with selector:" %)
                               true)
                            selectors)]
        ;; Switch back to main content
        (try
          (e/switch-frame-top driver)
          (catch Exception _))

        (if success
          (do
            (e/wait 2)
            (eprintln "Banner dismissed"))
          (eprintln "Could not dismiss banner - no selector matched"))))

    :linkme
    (do
      (eprintln "Attempting to dismiss LinkMe banner...")
      (when (try-selector driver {:xpath "//button[contains(text(), 'Accept All')]"} 5)
        (e/wait 2)
        (eprintln "Banner dismissed")))

    ;; No specific handling for other platforms
    (eprintln "No banner dismissal needed for platform:" platform)))

(defn scrape-bio-link
  [driver config username url]
  (try
    (e/go driver url)
    (e/wait 5)  ;; Wait longer for page and banner to fully load

    (let [platform        (identify-platform url)
          _               (dismiss-link-platform-banners driver platform)
          screenshot-file (screenshot-path (:data-dir config) username (name platform))
          cookies         (try
                            (into []
                                  (keep (fn [cookie]
                                          (when-let [name (get cookie "name")]
                                            {:name      name
                                             :domain    (get cookie "domain")
                                             :value     (get cookie "value")
                                             :http-only (get cookie "httpOnly" false)
                                             :secure    (get cookie "secure" false)})))
                                  (e/get-cookies driver))
                            (catch Exception _ []))

          outbound-links (try
                           (let [js-result (e/js-execute driver
                                                         "return Array.from(document.querySelectorAll('a[href]')).map((a, idx) => ({
                                                           href: a.href,
                                                           text: a.textContent.trim(),
                                                           position: idx
                                                         }));")]
                             (eprintln "Found" (count js-result) "links via JavaScript")
                             (keep (fn [link-data]
                                     (let [href          (:href link-data)
                                           text          (:text link-data)
                                           idx           (:position link-data)
                                           unwrapped-url (unwrap-instagram-url href)]
                                       (when (and href (not (str/blank? href)))
                                         {:url      unwrapped-url
                                          :text     text
                                          :position idx
                                          :utm      (parse-utm-params unwrapped-url)})))
                                   js-result))
                           (catch Exception e
                             (eprintln "Error extracting links:" (.getMessage e))
                             []))

          header-links (try
                         (into #{}
                               (keep #(e/get-element-attr driver % :href)
                                     (e/query-all driver {:css "header a[href], nav a[href]"})))
                         (catch Exception _ #{}))

          footer-links (try
                         (into #{}
                               (keep #(e/get-element-attr driver % :href)
                                     (e/query-all driver {:css "footer a[href]"})))
                         (catch Exception _ #{}))

          ;; Enrich outbound links with destination screenshots
          enriched-links (doall
                          (keep (fn [link]
                                  (Thread/sleep (:rate-limit-ms config))
                                  (if-let [dest (scrape-destination driver config username
                                                                    (:url link) (:text link))]
                                    (assoc link :destination dest)
                                    link))
                                outbound-links))]

      (full-page-screenshot driver screenshot-file)

      {:url            url
       :platform       platform
       :screenshot     screenshot-file
       :cookies        cookies
       :outbound-links enriched-links
       :header-links   header-links
       :footer-links   footer-links})
    (catch Exception e
      (println "Error scraping bio link" url ":" (.getMessage e))
      nil)))

(defn scrape-profile
  [driver cookies config url]
  (eprintln "Scraping profile:" url)
  (when-let [ig-data (scrape-instagram-profile driver cookies config url)]
    (let [username          (:instagram/username ig-data)
          bio-link-analysis (doall
                             (keep (fn [bio-link]
                                     (Thread/sleep (:rate-limit-ms config))
                                     (scrape-bio-link driver config username bio-link))
                                   (:instagram/bio-links ig-data)))]
      (merge ig-data
             {:scraped-at        (java.util.Date.)
              :bio-link-analysis bio-link-analysis}))))

;;; ----------------------------------------------------------------------------
;;; Output

(defn remove-empty-colls
  "Recursively remove empty collections from nested data structures"
  [m]
  (cond
    (map? m)
    (into {}
          (keep (fn [[k v]]
                  (let [cleaned (remove-empty-colls v)]
                    (when-not (or (and (coll? cleaned) (empty? cleaned))
                                  (nil? cleaned))
                      [k cleaned])))
                m))

    (sequential? m)
    (into (empty m)
          (keep (fn [v]
                  (let [cleaned (remove-empty-colls v)]
                    (when-not (and (coll? cleaned) (empty? cleaned))
                      cleaned)))
                m))

    :else m))

(defn save-result
  [data-dir username result]
  (let [scraped-dir (str data-dir "/scraped")
        date-str    (.format (java.text.SimpleDateFormat. "yyyy-MM-dd")
                             (:scraped-at result))
        output-file (str scraped-dir "/" username "-" date-str ".edn")
        cleaned     (remove-empty-colls result)]
    (fs/create-dirs scraped-dir)
    (with-open [w (io/writer output-file)]
      (pprint/pprint cleaned w))
    (println "Saved results to" output-file)))

;;; ----------------------------------------------------------------------------
;;; Main

(defn -main [& args]
  (when (empty? args)
    (eprintln "Usage: bb src/scrape.clj <file-or-url>")
    (eprintln "  file-or-url: path to EDN file with profile URLs, or a single Instagram URL")
    (System/exit 1))

  (let [input         (first args)
        config        (load-config)
        profiles      (if (str/starts-with? input "http")
                        [input]  ;; Single URL
                        (-> input slurp edn/read-string))  ;; File with URLs
        driver-opts   {:driver   (:browser config)
                       :headless (:headless config)}]

    (when (empty? profiles)
      (eprintln "No profiles found")
      (System/exit 1))

    (eprintln "Starting scraper with" (count profiles) "profile(s)")
    (when-not (str/starts-with? input "http")
      (eprintln "Using profiles from:" input))
    (eprintln "Extracting cookies from Firefox profile...")

    (let [cookies (extract-instagram-cookies firefox-profile-path)]
      (eprintln "Found" (count cookies) "Instagram cookies")
      (eprintln "Config:" config)

      (e/with-driver (:browser config) driver-opts driver
        (doseq [profile-url profiles]
          (when-let [result (scrape-profile driver cookies config profile-url)]
            (save-result (:data-dir config)
                         (:instagram/username result)
                         result)
            (Thread/sleep (:rate-limit-ms config)))))

      (eprintln "Scraping complete"))))

(apply -main *command-line-args*)
