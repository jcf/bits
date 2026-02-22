(ns bits.ledger
  (:require
   [clojure.spec.alpha :as s]
   [datomic.api :as d]
   [java-time.api :as time]))

;;; ----------------------------------------------------------------------------
;;; Queries

(def account-by-code-query
  '[:find (pull ?a [*]) .
    :in $ ?code
    :where
    [?a :ledger-account/code ?code]])

(def account-debits-query
  '[:find (sum ?amount) .
    :in $ ?account-eid
    :where
    [?posting :posting/account ?account-eid]
    [?posting :posting/direction :posting.direction/debit]
    [?posting :posting/amount ?amount]
    [?je :journal-entry/postings ?posting]
    [?je :journal-entry/status :journal-entry.status/posted]])

(def account-credits-query
  '[:find (sum ?amount) .
    :in $ ?account-eid
    :where
    [?posting :posting/account ?account-eid]
    [?posting :posting/direction :posting.direction/credit]
    [?posting :posting/amount ?amount]
    [?je :journal-entry/postings ?posting]
    [?je :journal-entry/status :journal-entry.status/posted]])

;;; ----------------------------------------------------------------------------
;;; Balance calculation

(def debit-normal-types
  #{:ledger-account.type/asset :ledger-account.type/expense})

(defn calculate-balance
  [account-type debits credits]
  (if (debit-normal-types account-type)
    (- (or debits 0) (or credits 0))
    (- (or credits 0) (or debits 0))))

;;; ----------------------------------------------------------------------------
;;; Validation (convenience — early feedback, not source of truth)

(defn validate-balanced
  [postings]
  (let [sum-by (fn [dir]
                 (->> postings
                      (filter #(= dir (:posting/direction %)))
                      (map :posting/amount)
                      (reduce + 0)))
        debits  (sum-by :posting.direction/debit)
        credits (sum-by :posting.direction/credit)]
    (when (not= debits credits)
      {:error   :unbalanced
       :debits  debits
       :credits credits})))

;;; ----------------------------------------------------------------------------
;;; Transaction data

(defn ledger-account-tx
  [{:keys [code name type currency description]
    :or   {description nil}}]
  (cond-> {:ledger-account/id         (d/squuid)
           :ledger-account/code       code
           :ledger-account/name       name
           :ledger-account/type       type
           :ledger-account/currency   currency
           :ledger-account/active?    true
           :ledger-account/created-at (time/java-date)}
    description (assoc :ledger-account/description description)))

(defn default-accounts-txes
  [currency]
  [(ledger-account-tx {:code     "asset:incoming-payments"
                       :name     "Incoming Payments"
                       :type     :ledger-account.type/asset
                       :currency currency})

   (ledger-account-tx {:code     "asset:platform-holdings"
                       :name     "Platform Holdings"
                       :type     :ledger-account.type/asset
                       :currency currency})

   (ledger-account-tx {:code     "liability:creator-balance"
                       :name     "Creator Balance"
                       :type     :ledger-account.type/liability
                       :currency currency})

   (ledger-account-tx {:code     "liability:buyer-refunds-due"
                       :name     "Buyer Refunds Due"
                       :type     :ledger-account.type/liability
                       :currency currency})

   (ledger-account-tx {:code     "revenue:platform-fees"
                       :name     "Platform Fees"
                       :type     :ledger-account.type/revenue
                       :currency currency})

   (ledger-account-tx {:code     "expense:processor-fees"
                       :name     "Processor Fees"
                       :type     :ledger-account.type/expense
                       :currency currency})

   (ledger-account-tx {:code     "expense:refunds-issued"
                       :name     "Refunds Issued"
                       :type     :ledger-account.type/expense
                       :currency currency})])
