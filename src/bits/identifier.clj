(ns bits.identifier
  (:import
   (java.util UUID)))

;;; ----------------------------------------------------------------------------
;;; Base36 encoding for UUIDs

(def ^:const ^:private radix 36)
(def ^:const ^:private encoded-length 25)

(defn- asl [a b] (.add (.shiftLeft b 64) a))

(defn encode
  [^UUID uuid]
  (let [msb (.getMostSignificantBits uuid)
        lsb (.getLeastSignificantBits uuid)
        hi  (BigInteger/valueOf msb)
        lo  (BigInteger/valueOf lsb)
        hi  (cond-> hi (neg? msb) (asl BigInteger/ONE))
        lo  (cond-> lo (neg? lsb) (asl BigInteger/ONE))
        n   (asl lo hi)
        s   (.toString n radix)]
    (str (subs "0000000000000000000000000" 0 (- encoded-length (count s))) s)))

(def ^:private mask-64
  (.subtract (.shiftLeft BigInteger/ONE 64) BigInteger/ONE))

(defn decode
  [^String s]
  (let [n   (BigInteger. s radix)
        lo  (.and n mask-64)
        hi  (.shiftRight n 64)
        lsb (.longValue lo)
        msb (.longValue hi)]
    (UUID. msb lsb)))

(defn parse
  [^String s]
  (try
    (case (count s)
      25 (decode s)
      36 (UUID/fromString s)
      nil)
    (catch Exception _ nil)))
