(ns bits.coerce
  (:require
   [bits.cryptex :as cryptex]
   [malli.core :as m]
   [malli.transform :as mt]
   [reitit.coercion.malli :as coercion.malli]))

;;; ----------------------------------------------------------------------------
;;; Sensitive data types
;;;
;;; These types wrap sensitive data in cryptex at the coercion boundary,
;;; preventing accidental logging or exposure in handlers.

(def Password
  "Password wrapped in cryptex. Cannot be printed or logged."
  [:fn {:decode/string cryptex/cryptex} cryptex/cryptex?])

(def Email
  "Email address wrapped in cryptex for PII protection."
  [:fn {:decode/string cryptex/cryptex} cryptex/cryptex?])

;;; ----------------------------------------------------------------------------
;;; Registry

(def registry
  {:password Password
   :email    Email})

;;; ----------------------------------------------------------------------------
;;; Transformer

(def sensitive-transformer
  "Transformer that wraps sensitive strings in cryptex."
  (mt/transformer
   {:name :sensitive
    :decoders {:password {:compile (fn [_ _] cryptex/cryptex)}
               :email    {:compile (fn [_ _] cryptex/cryptex)}}}))

(def string-transformer
  "Combined string + sensitive transformer for form parameters."
  (mt/transformer
   mt/string-transformer
   sensitive-transformer))

;;; ----------------------------------------------------------------------------
;;; Coercion

(def coercion
  "Malli coercion with sensitive data handling for form parameters.
   Use this for routes that accept email/password."
  (coercion.malli/create
   {:transformers {:body     {:default coercion.malli/default-transformer-provider}
                   :string   {:default coercion.malli/string-transformer-provider}
                   :response {:default coercion.malli/default-transformer-provider}
                   :form     {:default (constantly string-transformer)}}
    :options      {:registry (merge (m/default-schemas) registry)}}))
