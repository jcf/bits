(ns bits.test.browser
  (:require
   [babashka.fs :as fs]
   [bits.test.app :as t]
   [clojure.pprint :as pprint]
   [etaoin.api :as e]
   [io.pedestal.log :as log]
   [java-time.api :as time]
   [lambdaisland.uri :as uri]))

;;; ----------------------------------------------------------------------------
;;; Driver lifecycle

(def ^:const ^:private session-dir "target/browser-sessions")

(defrecord Driver [etaoin service])

(defmethod print-method Driver
  [_ ^java.io.Writer w]
  (.write w "#<Driver>"))

(defmethod pprint/simple-dispatch Driver
  [_]
  (pr "#<Driver>"))

(defn- ->etaoin  [driver] (:etaoin driver))
(defn- ->service [driver] (:service driver))

(defn make-driver
  [service]
  (->Driver (e/firefox {:headless true}) service))

(defn quit
  [driver]
  (e/quit (->etaoin driver)))

;;; ----------------------------------------------------------------------------
;;; Navigation

(defn goto
  [driver path]
  (e/go (->etaoin driver) (t/service-url (->service driver) path)))

(defn current-path
  [driver]
  (-> (e/get-url (->etaoin driver)) uri/uri :path))

;;; ----------------------------------------------------------------------------
;;; Selectors

(defn- ->query
  [selector]
  (cond
    (keyword? selector) {:name (name selector)}
    (string? selector)  {:css selector}
    :else               selector))

;;; ----------------------------------------------------------------------------
;;; Forms

(defn fill
  [driver field-name value]
  (e/fill (->etaoin driver) {:name (name field-name)} value))

(defn wait-to-fill
  [driver selector value]
  (let [e (->etaoin driver)
        q (->query selector)]
    (e/wait-visible e q)
    (e/fill e q value)))

(defn click
  [driver selector]
  (e/click (->etaoin driver) (->query selector)))

(defn submit
  [driver selector]
  (e/submit (->etaoin driver) (->query selector)))

;;; ----------------------------------------------------------------------------
;;; Queries

(defn text
  [driver selector]
  (e/get-element-text (->etaoin driver) (->query selector)))

(defn visible?
  [driver selector]
  (e/visible? (->etaoin driver) (->query selector)))

(defn exists?
  [driver selector]
  (e/exists? (->etaoin driver) (->query selector)))

;;; ----------------------------------------------------------------------------
;;; Wait

(defn wait-to-click
  [driver selector]
  (let [e (->etaoin driver)]
    (e/wait-visible e selector)
    (e/click e selector)))

(defn wait-visible
  [driver selector]
  (e/wait-visible (->etaoin driver) selector))

;;; ----------------------------------------------------------------------------
;;; Debug

(defn- get-source
  [driver]
  (e/get-source (->etaoin driver)))

(defn screenshot
  ([driver]
   (let [ts (time/format "yyyyMMdd-HHmmssSSS" (time/local-date-time))]
     (screenshot driver (str "screenshot-" ts ".png"))))
  ([driver path]
   (log/debug :msg "Capturing screenshot..." :path path)
   (e/screenshot (->etaoin driver) path)))

;;; ----------------------------------------------------------------------------
;;; Helpful Driver

(defn with-driver*
  [service body-fn]
  (let [driver (make-driver service)]
    (try
      (body-fn driver)
      (catch Throwable t
        (let [ts  (time/format "yyyyMMdd-HHmmssSSS" (time/local-date-time))
              dir (fs/file session-dir ts)]
          (fs/create-dirs dir)
          (screenshot driver (str (fs/file dir "screenshot.png")))
          (spit (fs/file dir "page-source.html") (get-source driver))
          (spit (fs/file dir "console.edn")
                (with-out-str
                  (pprint/pprint (e/get-logs (->etaoin driver)))))
          (throw (ex-info "Browser session failed?!"
                          {:dir       (str dir)
                           :timestamp ts}
                          t))))
      (finally
        (quit driver)))))

(defmacro with-driver
  [[binding service] & body]
  `(with-driver* ~service (^:once fn* [~binding] ~@body)))
