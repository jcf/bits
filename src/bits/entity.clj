(ns bits.entity
  "Peer-side value and entity specs for Datomic data.

  Attribute-level specs capture format/value invariants we cannot cheaply
  enforce in the transactor (attribute predicates require user code on the
  transactor classpath). Entity-level `s/keys` specs describe the shape of
  whole entity maps — use them in `make-*` factories via `:pre` or `s/assert`
  so validation fails at the call site, not at `d/transact` time.

  Structural invariants — types, cardinality, uniqueness, required attributes
  (via `:db.entity/attrs` + `:db/ensure`) — live in bits.schema and are
  enforced by Datomic itself."
  (:require
   [clojure.spec.alpha :as s]
   [clojure.string :as str]))

(defn present?
  [s]
  (and (string? s) (not (str/blank? s))))

;;; ----------------------------------------------------------------------------
;;; Money

(s/def :money/amount pos-int?)
(s/def :money/currency keyword?)

;;; ----------------------------------------------------------------------------
;;; Product

(s/def :product/title present?)

;;; ----------------------------------------------------------------------------
;;; Variant

(s/def :variant/name present?)

;;; ----------------------------------------------------------------------------
;;; SKU

(s/def :sku/code present?)

;;; ----------------------------------------------------------------------------
;;; Ledger account

(s/def :ledger-account/code present?)
(s/def :ledger-account/name present?)

;;; ----------------------------------------------------------------------------
;;; Posting

(s/def :posting/account uuid?)
(s/def :posting/amount pos-int?)
(s/def :posting/direction #{:posting.direction/credit :posting.direction/debit})

;;; ----------------------------------------------------------------------------
;;; Receipt

(s/def :receipt/external-id present?)
(s/def :receipt/idempotency-key present?)

;;; ----------------------------------------------------------------------------
;;; Line item

(s/def :line-item/product-title present?)
(s/def :line-item/variant-name present?)
(s/def :line-item/sku-code present?)
