(ns bits.schema)

;;; ----------------------------------------------------------------------------
;;; User

(def user-schema
  [{:db/ident       :user/id
    :db/valueType   :db.type/uuid
    :db/cardinality :db.cardinality/one
    :db/unique      :db.unique/identity}

   {:db/ident       :user/email
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one
    :db/unique      :db.unique/identity}

   {:db/ident       :user/password-hash
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one}

   {:db/ident       :user/created-at
    :db/valueType   :db.type/instant
    :db/cardinality :db.cardinality/one}])

;;; ----------------------------------------------------------------------------
;;; Tenant

(def tenant-schema
  [{:db/ident       :tenant/id
    :db/valueType   :db.type/uuid
    :db/cardinality :db.cardinality/one
    :db/unique      :db.unique/identity}

   {:db/ident       :tenant/purpose
    :db/valueType   :db.type/keyword
    :db/cardinality :db.cardinality/one
    :db/unique      :db.unique/identity}

   {:db/ident       :tenant/created-at
    :db/valueType   :db.type/instant
    :db/cardinality :db.cardinality/one}

   {:db/ident       :tenant/domains
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/many}])

;;; ----------------------------------------------------------------------------
;;; Domain

(def domain-schema
  [{:db/ident       :domain/name
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one
    :db/unique      :db.unique/identity}])

;;; ----------------------------------------------------------------------------
;;; Creator

(def creator-schema
  [{:db/ident       :creator/handle
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one
    :db/unique      :db.unique/identity}

   {:db/ident       :creator/display-name
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one}

   {:db/ident       :creator/bio
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one}

   {:db/ident       :creator/avatar-url
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one}

   {:db/ident       :creator/banner-url
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one}

   {:db/ident       :creator/links
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/many
    :db/isComponent true}

   {:db/ident       :creator/posts
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/many}

   {:db/ident       :link/label
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one}

   {:db/ident       :link/url
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one}

   {:db/ident       :link/icon
    :db/valueType   :db.type/keyword
    :db/cardinality :db.cardinality/one}])

;;; ----------------------------------------------------------------------------
;;; Post

(def post-schema
  [{:db/ident       :post/id
    :db/valueType   :db.type/uuid
    :db/cardinality :db.cardinality/one
    :db/unique      :db.unique/identity}

   {:db/ident       :post/text
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one}

   {:db/ident       :post/created-at
    :db/valueType   :db.type/instant
    :db/cardinality :db.cardinality/one}

   {:db/ident       :post/image-url
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one}])

;;; ----------------------------------------------------------------------------
;;; Membership

(def membership-schema
  [{:db/ident       :membership/id
    :db/valueType   :db.type/uuid
    :db/cardinality :db.cardinality/one
    :db/unique      :db.unique/identity}

   {:db/ident       :membership/user
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/one}

   {:db/ident       :membership/tenant
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/one}

   {:db/ident       :membership/role
    :db/valueType   :db.type/keyword
    :db/cardinality :db.cardinality/one}])

;;; ----------------------------------------------------------------------------
;;; Shop Idents

(def shop-ident-schema
  [;; Product status
   {:db/ident :product.status/draft}
   {:db/ident :product.status/active}
   {:db/ident :product.status/archived}

   ;; Variant type (fulfilment)
   {:db/ident :variant.type/digital}
   {:db/ident :variant.type/physical}

   ;; Ledger account types — the five fundamental categories (Pacioli)
   ;; Debit-normal (asset, expense): debits increase, credits decrease
   ;; Credit-normal (liability, equity, revenue): credits increase, debits decrease
   {:db/ident :ledger-account.type/asset}
   {:db/ident :ledger-account.type/liability}
   {:db/ident :ledger-account.type/equity}
   {:db/ident :ledger-account.type/revenue}
   {:db/ident :ledger-account.type/expense}

   ;; Posting direction
   {:db/ident :posting.direction/debit}
   {:db/ident :posting.direction/credit}

   ;; Journal entry status (Modern Treasury lifecycle)
   ;; pending → posted (confirmed) or pending → archived (failed)
   ;; posted entries are never mutated — create reversing entries
   {:db/ident :journal-entry.status/pending}
   {:db/ident :journal-entry.status/posted}
   {:db/ident :journal-entry.status/archived}

   ;; Payment processor
   {:db/ident :processor/stripe}
   {:db/ident :processor/monero}

   ;; Currencies (ISO 4217) — add as needed per tenant
   {:db/ident :currency/GBP}
   {:db/ident :currency/USD}
   {:db/ident :currency/EUR}

   ;; Jurisdictions (ISO 3166-1 alpha-2) — add as needed
   {:db/ident :jurisdiction/GB}
   {:db/ident :jurisdiction/US}
   {:db/ident :jurisdiction/DE}
   {:db/ident :jurisdiction/FR}])

;;; ----------------------------------------------------------------------------
;;; Money (component entity)
;;;
;;; Amount (long, minor units) + currency (ref to ident). Used by :variant/price
;;; and :line-item/unit-price. NOT used by postings — posting currency determined
;;; by the account (Formance pattern).

(def money-schema
  [{:db/ident       :money/amount
    :db/valueType   :db.type/long
    :db/cardinality :db.cardinality/one
    :db.attr/preds  'clojure.core/pos-int?
    :db/doc         "Amount in minor currency units (pence/cents). £9.99 = 999.
                     Always positive."}

   {:db/ident       :money/currency
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/one
    :db/doc         "Ref to a :currency/* ident. ISO 4217."}])

;;; ----------------------------------------------------------------------------
;;; Product

(def product-schema
  [{:db/ident       :product/id
    :db/valueType   :db.type/uuid
    :db/cardinality :db.cardinality/one
    :db/unique      :db.unique/identity
    :db/doc         "Unique identifier for this product."}

   {:db/ident       :product/title
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one
    :db.attr/preds  'bits.string/present?
    :db/doc         "Display title of the product."}

   {:db/ident       :product/description
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one
    :db/doc         "Long-form description. Stored as markdown, rendered to Hiccup."}

   {:db/ident       :product/media
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/many
    :db/doc         "References to media entities (images, video)."}

   {:db/ident       :product/variants
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/many
    :db/doc         "The purchasable configurations of this product."}

   {:db/ident       :product/position
    :db/valueType   :db.type/long
    :db/cardinality :db.cardinality/one
    :db.attr/preds  'clojure.core/pos-int?
    :db/doc         "Manual sort order for shop grid display."}

   {:db/ident       :product/status
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/one
    :db/doc         "Lifecycle status. Ref to a :product.status/* ident."}

   {:db/ident       :product/created-at
    :db/valueType   :db.type/instant
    :db/cardinality :db.cardinality/one
    :db/doc         "When this product was created. Used for 'newest' sort."}])

;;; ----------------------------------------------------------------------------
;;; Variant
;;;
;;; Immutable once sold. Price change means deactivating and creating a new variant.

(def variant-schema
  [{:db/ident       :variant/id
    :db/valueType   :db.type/uuid
    :db/cardinality :db.cardinality/one
    :db/unique      :db.unique/identity
    :db/doc         "Unique identifier for this variant."}

   {:db/ident       :variant/name
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one
    :db.attr/preds  'bits.string/present?
    :db/doc         "Display name, e.g. 'A3 Wall Calendar' or 'Digital Download'."}

   {:db/ident       :variant/sku
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/one
    :db/isComponent true
    :db/doc         "Component entity representing the structured SKU."}

   {:db/ident       :variant/price
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/one
    :db/isComponent true
    :db/doc         "The price of this variant. Component Money entity."}

   {:db/ident       :variant/type
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/one
    :db/doc         "Fulfilment type. Ref to a :variant.type/* ident."}

   {:db/ident       :variant/quantity-limit
    :db/valueType   :db.type/long
    :db/cardinality :db.cardinality/one
    :db.attr/preds  'clojure.core/pos-int?
    :db/doc         "Maximum units available. Absent means unlimited (typical for digital)."}

   {:db/ident       :variant/active?
    :db/valueType   :db.type/boolean
    :db/cardinality :db.cardinality/one
    :db/doc         "Whether this variant is currently purchasable."}

   {:db/ident       :variant/media
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/many
    :db/doc         "Variant-specific media, e.g. a photo of the A3 vs A4 version."}

   {:db/ident       :variant/created-at
    :db/valueType   :db.type/instant
    :db/cardinality :db.cardinality/one
    :db/doc         "When this variant was created."}])

;;; ----------------------------------------------------------------------------
;;; SKU (component of Variant)

(def sku-schema
  [{:db/ident       :sku/code
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one
    :db/unique      :db.unique/identity
    :db.attr/preds  'bits.string/present?
    :db/doc         "Creator-assigned SKU code. Unique within the tenant database."}

   {:db/ident       :sku/gtin
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one
    :db/doc         "GS1 Global Trade Item Number. Optional. For physical goods."}])

;;; ----------------------------------------------------------------------------
;;; Ledger Account
;;;
;;; Named bucket that tracks a derived balance. Balances are computed (Σ postings),
;;; never stored.

(def ledger-account-schema
  [{:db/ident       :ledger-account/id
    :db/valueType   :db.type/uuid
    :db/cardinality :db.cardinality/one
    :db/unique      :db.unique/identity
    :db/doc         "Unique identifier for this account."}

   ;; FIXME Avoid stringly typed entities - use `:db/ident`.
   {:db/ident       :ledger-account/code
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one
    :db/unique      :db.unique/identity
    :db.attr/preds  'bits.string/present?
    :db/doc         "Hierarchical account code, e.g. 'revenue:platform-fees'.
                     The primary human-readable identifier. Unique within tenant."}

   {:db/ident       :ledger-account/name
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one
    :db.attr/preds  'bits.string/present?
    :db/doc         "Human-friendly display name, e.g. 'Platform Fees'."}

   {:db/ident       :ledger-account/type
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/one
    :db/doc         "Account classification. Ref to :ledger-account.type/* ident.
                     Determines normal balance direction (debit or credit normal)."}

   {:db/ident       :ledger-account/currency
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/one
    :db/doc         "Currency denomination. Ref to :currency/* ident.
                     All postings to this account must be in this currency.
                     Multi-currency requires separate accounts per currency."}

   {:db/ident       :ledger-account/description
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one
    :db/doc         "Optional description of the account's purpose."}

   {:db/ident       :ledger-account/active?
    :db/valueType   :db.type/boolean
    :db/cardinality :db.cardinality/one
    :db/doc         "Whether this account accepts new postings. Inactive accounts
                     retain their history but reject new entries."}

   {:db/ident       :ledger-account/created-at
    :db/valueType   :db.type/instant
    :db/cardinality :db.cardinality/one
    :db/doc         "When this account was created."}])

;;; ----------------------------------------------------------------------------
;;; Journal Entry
;;;
;;; Atomic, balanced financial event. Immutable once posted — corrections are
;;; reversing entries.

(def journal-entry-schema
  [{:db/ident       :journal-entry/id
    :db/valueType   :db.type/uuid
    :db/cardinality :db.cardinality/one
    :db/unique      :db.unique/identity
    :db/doc         "Unique identifier for this journal entry."}

   {:db/ident       :journal-entry/description
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one
    :db/doc         "Human-readable description of what this entry records.
                     E.g. 'Purchase: A3 Wall Calendar'. Must not contain PII —
                     use pseudonymous donor labels, not emails or names."}

   {:db/ident       :journal-entry/status
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/one
    :db/doc         "Lifecycle status. Ref to :journal-entry.status/* ident.
                     Only posted entries affect account balances."}

   {:db/ident       :journal-entry/postings
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/many
    :db/isComponent true
    :db/doc         "The set of postings comprising this entry. Component entities.
                     Must contain at least two postings. Must sum to zero
                     (debits = credits). Enforced by transaction function."}

   {:db/ident       :journal-entry/effective-at
    :db/valueType   :db.type/instant
    :db/cardinality :db.cardinality/one
    :db/doc         "When this financial event occurred (business date).
                     May differ from created-at (the recording date)."}

   {:db/ident       :journal-entry/created-at
    :db/valueType   :db.type/instant
    :db/cardinality :db.cardinality/one
    :db/doc         "When this entry was recorded in the ledger."}

   {:db/ident       :journal-entry/receipt
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/one
    :db/isComponent true
    :db/doc         "Evidence from the external system that triggered this entry.
                     Component entity. Absent for internal entries (fee splits,
                     adjustments). A Stripe webhook response and a journal entry
                     are different things — the receipt is the evidence, the
                     journal entry is the accounting record it produced."}

   {:db/ident       :journal-entry/reverses
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/one
    :db/doc         "If this entry is a correction/reversal, references the
                     original journal entry being reversed. Ref, not component."}])

;;; ----------------------------------------------------------------------------
;;; Posting (component of Journal Entry)
;;;
;;; Single debit or credit. Amount is always positive; direction is separate.
;;; Currency from the account.

(def posting-schema
  [{:db/ident       :posting/account
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/one
    :db/doc         "The ledger account this posting affects."}

   {:db/ident       :posting/amount
    :db/valueType   :db.type/long
    :db/cardinality :db.cardinality/one
    :db.attr/preds  'clojure.core/pos-int?
    :db/doc         "Amount in minor currency units. Always positive.
                     Currency determined by the account's currency. £9.99 = 999."}

   {:db/ident       :posting/direction
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/one
    :db/doc         "Whether this is a debit or credit. Ref to
                     :posting.direction/* ident."}])

;;; ----------------------------------------------------------------------------
;;; Receipt (component of Journal Entry)
;;;
;;; Evidence from the external system. Extensible via :receipt.stripe/*,
;;; :receipt.monero/* namespaces per tenant.

(def receipt-schema
  [{:db/ident       :receipt/processor
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/one
    :db/doc         "Which processor produced this receipt. Ref to :processor/* ident."}

   {:db/ident       :receipt/external-id
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one
    :db.attr/preds  'bits.string/present?
    :db/doc         "The processor's identifier for this event. Stripe PaymentIntent
                     ID, Monero tx hash, etc. Used for reconciliation."}

   {:db/ident       :receipt/idempotency-key
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one
    :db/unique      :db.unique/value
    :db.attr/preds  'bits.string/present?
    :db/doc         "Prevents duplicate journal entries from webhook retries.
                     Unique within tenant database. Typically derived from the
                     processor event: e.g. 'stripe:{intent-id}:{event-type}'."}

   {:db/ident       :receipt/received-at
    :db/valueType   :db.type/instant
    :db/cardinality :db.cardinality/one
    :db/doc         "When we received this from the processor."}])

;;; ----------------------------------------------------------------------------
;;; Line Item
;;;
;;; Bridge between catalog and ledger. Commercial record with denormalised
;;; snapshots for legal defensibility and excision survival.

(def line-item-schema
  [{:db/ident       :line-item/id
    :db/valueType   :db.type/uuid
    :db/cardinality :db.cardinality/one
    :db/unique      :db.unique/identity
    :db/doc         "Unique identifier for this line item."}

   {:db/ident       :line-item/variant
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/one
    :db/doc         "Reference to the variant that was purchased."}

   {:db/ident       :line-item/buyer
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/one
    :db/doc         "The user who made this purchase. Only PII ref in the schema."}

   {:db/ident       :line-item/journal-entry
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/one
    :db/doc         "The journal entry that recorded this purchase in the ledger."}

   {:db/ident       :line-item/quantity
    :db/valueType   :db.type/long
    :db/cardinality :db.cardinality/one
    :db.attr/preds  'clojure.core/pos-int?
    :db/doc         "Number of units purchased."}

   ;; Denormalised snapshots — frozen at purchase time

   {:db/ident       :line-item/product-title
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one
    :db.attr/preds  'bits.string/present?
    :db/doc         "Snapshot of product title at time of purchase."}

   {:db/ident       :line-item/variant-name
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one
    :db.attr/preds  'bits.string/present?
    :db/doc         "Snapshot of variant name at time of purchase."}

   {:db/ident       :line-item/sku-code
    :db/valueType   :db.type/string
    :db/cardinality :db.cardinality/one
    :db.attr/preds  'bits.string/present?
    :db/doc         "Snapshot of SKU code at time of purchase."}

   {:db/ident       :line-item/unit-price
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/one
    :db/isComponent true
    :db/doc         "Snapshot of price at time of purchase. Component Money entity.
                     Survives variant deactivation and entity excision."}

   {:db/ident       :line-item/buyer-jurisdiction
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/one
    :db/doc         "Buyer's jurisdiction at purchase time. Ref to :jurisdiction/* ident. For VAT."}

   {:db/ident       :line-item/created-at
    :db/valueType   :db.type/instant
    :db/cardinality :db.cardinality/one
    :db/doc         "When this purchase was made."}])

;;; ----------------------------------------------------------------------------
;;; Tenant Ownership (Shop)

(def tenant-shop-schema
  [{:db/ident       :tenant/products
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/many
    :db/doc         "Products belonging to this tenant's shop."}

   {:db/ident       :tenant/line-items
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/many
    :db/doc         "All purchases made within this tenant."}

   {:db/ident       :tenant/ledger-accounts
    :db/valueType   :db.type/ref
    :db/cardinality :db.cardinality/many
    :db/doc         "Ledger accounts belonging to this tenant's financial system."}])

;;; ----------------------------------------------------------------------------
;;; Full schema

(def schema
  (->> [user-schema
        tenant-schema
        domain-schema
        creator-schema
        post-schema
        membership-schema
        shop-ident-schema
        money-schema
        product-schema
        variant-schema
        sku-schema
        ledger-account-schema
        journal-entry-schema
        posting-schema
        receipt-schema
        line-item-schema
        tenant-shop-schema]
       (reduce into)))
