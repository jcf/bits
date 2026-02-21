(ns bits.locale
  (:require
   [mr-worldwide.core :as i18n]
   [ring.util.response :as response])
  (:import
   (java.util Locale
              Locale$LanguageRange)))

;;; ----------------------------------------------------------------------------
;;; Locale

(defn string->locale
  [^String s]
  {:pre [(string? s)]}
  (Locale. s))

;;; ----------------------------------------------------------------------------
;;; HTTP

(defn parse-accept-language
  [s]
  (when (some? s)
    (try
      (mapv #(Locale/forLanguageTag (.getRange %))
            (Locale$LanguageRange/parse ^String s))
      (catch IllegalArgumentException _
        []))))

(defn lookup-locale
  [default-locale supported-locales s]
  (let [ranges (seq (Locale$LanguageRange/parse (or s "")))]
    (or (and (seq ranges) (Locale/lookup ranges supported-locales))
        default-locale)))

(defn request->locale
  [request default-locale supported-locales]
  (or (:session/locale request)
      (lookup-locale default-locale
                     supported-locales
                     (response/get-header request "accept-language"))))

;;; ----------------------------------------------------------------------------
;;; Translation

(def ^:dynamic *locale* i18n/*user-locale*)

(defmacro with-locale
  [locale & body]
  `(binding [i18n/*user-locale* ~locale]
     ~@body))

(defmacro tru
  [format-string & args]
  `(i18n/tru ~format-string ~@args))

(defmacro trs
  [format-string & args]
  `(i18n/trs ~format-string ~@args))
