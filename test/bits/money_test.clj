(ns bits.money-test
  (:require
   [bits.money :as sut]
   [bits.string :as string]
   [clojure.test :refer [are deftest is]])
  (:import
   (java.util Locale)))

;;; ----------------------------------------------------------------------------
;;; Enrich

(deftest enrich
  (let [raw {:money/amount   999
             :money/currency {:db/ident :currency/GBP}}
        m   (sut/enrich raw)]
    (is (sut/currency? (::sut/iso m)))
    (is (= {:db/ident :currency/GBP} (:money/currency m)))
    (is (= 999 (:money/amount m)))))

;;; ----------------------------------------------------------------------------
;;; Format

(deftest format-price
  (let [fmt (fn [locale amount currency]
              (sut/format-price
               locale
               (sut/enrich {:money/amount   amount
                            :money/currency {:db/ident currency}})))]
    (are [locale amount currency expected]
         (= expected (fmt locale amount currency))
      Locale/FRANCE  100   :currency/EUR (str "1,00" string/nbsp "€")
      Locale/FRANCE  500   :currency/USD (str "5,00" string/nbsp "$US")
      Locale/FRANCE  999   :currency/GBP (str "9,99" string/nbsp "£GB")
      Locale/GERMANY 100   :currency/EUR (str "1,00" string/nbsp "€")
      Locale/GERMANY 500   :currency/USD (str "5,00" string/nbsp "$")
      Locale/GERMANY 999   :currency/GBP (str "9,99" string/nbsp "£")
      Locale/UK      1     :currency/GBP "£0.01"
      Locale/UK      100   :currency/EUR "€1.00"
      Locale/UK      10000 :currency/USD "US$100.00"
      Locale/UK      1499  :currency/GBP "£14.99"
      Locale/UK      500   :currency/USD "US$5.00"
      Locale/UK      999   :currency/GBP "£9.99"
      Locale/US      100   :currency/EUR "€1.00"
      Locale/US      500   :currency/USD "$5.00"
      Locale/US      999   :currency/GBP "£9.99")))
