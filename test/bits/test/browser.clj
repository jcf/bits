(ns bits.test.browser
  (:require
   [bits.test.app :as t]
   [clojure.pprint :as pprint]
   [etaoin.api :as e]
   [io.pedestal.log :as log]
   [java-time.api :as time]
   [lambdaisland.uri :as uri]))

;;; ----------------------------------------------------------------------------
;;; Driver lifecycle

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

(defmacro with-driver
  [[binding service] & body]
  `(let [~binding (make-driver ~service)]
     (try
       ~@body
       (finally
         (quit ~binding)))))

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

(defn get-source
  [driver]
  (e/get-source (->etaoin driver)))

(defn screenshot
  ([driver]
   (let [ts (time/format "yyyyMMdd-HHmmssSSS" (time/local-date-time))]
     (screenshot driver (str "screenshot-" ts ".png"))))
  ([driver path]
   (log/debug :msg "Capturing screenshot..." :path path)
   (e/screenshot (->etaoin driver) path)))
