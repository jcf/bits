(ns bits.lender-test
  (:require
   [bits.lender :as sut]
   [clojure.test :refer [deftest is]]))

(defn- recording-lender
  [{:keys [made released]}]
  (let [cache (sut/make-cache {:max-size 8})]
    {:cache cache
     :lend  (fn [k func]
              (sut/lend cache
                        (fn open [k'] [k' (count (swap! made conj k'))])
                        (fn record [_k resource] (swap! released conj resource))
                        k func))}))

;;; ----------------------------------------------------------------------------
;;; lend

(deftest lend
  (let [made           (atom [])
        released       (atom [])
        {:keys [lend]} (recording-lender {:made made :released released})]
    (is (= [:a 1] (lend :a identity)))
    (is (= [:a 1] (lend :a identity)) "Resource is reused across lends.")
    (is (= [:a] @made))
    (is (= [] @released))))

(deftest lend-requires-a-cache
  (is (thrown-with-msg? clojure.lang.ExceptionInfo #"without a cache"
                        (sut/lend nil identity (fn discard [_k _resource] nil)
                                  :a identity))))

;;; ----------------------------------------------------------------------------
;;; evict!

(deftest evict!
  (let [made                 (atom [])
        released             (atom [])
        {:keys [cache lend]} (recording-lender {:made made :released released})]
    (lend :a identity)
    (sut/evict! cache :a)
    (is (= [[:a 1]] @released) "Idle entry releases immediately on eviction.")
    (is (= [:a 2] (lend :a identity)) "Next lend makes a fresh resource.")))

(deftest evict!-defers-release-while-borrowed
  (let [made                 (atom [])
        released             (atom [])
        {:keys [cache lend]} (recording-lender {:made made :released released})
        gate                 (promise)
        holding              (promise)
        borrower             (future (lend :a (fn hold [resource]
                                                (deliver holding resource)
                                                @gate
                                                resource)))]
    @holding
    (sut/evict! cache :a)
    (is (= [] @released) "Release waits for the borrower to check in.")
    (deliver gate :done)
    (is (= [:a 1] @borrower))
    (is (= [[:a 1]] @released))))

;;; ----------------------------------------------------------------------------
;;; drain!

(deftest drain!
  (let [made                 (atom [])
        released             (atom [])
        {:keys [cache lend]} (recording-lender {:made made :released released})]
    (lend :a identity)
    (lend :b identity)
    (sut/drain! cache)
    (is (= #{[:a 1] [:b 2]} (set @released)))))

;;; ----------------------------------------------------------------------------
;;; Concurrency

(deftest lend-races-evict!
  (let [made                 (atom [])
        released             (atom [])
        {:keys [cache lend]} (recording-lender {:made made :released released})
        errors               (atom 0)
        workers              (doall
                              (for [_ (range 16)]
                                (future
                                  (dotimes [_ 250]
                                    (try
                                      (lend :k (fn check [resource]
                                                 (when-not resource
                                                   (swap! errors inc))))
                                      (catch Exception _ (swap! errors inc)))))))
        evictor              (future (dotimes [_ 100] (sut/evict! cache :k)))]
    (run! deref workers)
    @evictor
    (sut/drain! cache)
    (is (zero? @errors))
    (is (= (count @made) (count @released))
        "Every resource made is released exactly once.")))
