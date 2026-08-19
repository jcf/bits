(ns bits.async-test
  (:require
   [bits.async :as sut]
   [clojure.test :refer [deftest is]])
  (:import
   (java.util.concurrent CancellationException ExecutionException)))

(deftest vfuture
  (is (= 42 @(sut/vfuture 42)))
  (let [ex (try @(sut/vfuture (throw (ex-info "boom" {})))
                (catch ExecutionException ex ex))]
    (is (= "boom" (ex-message (ex-cause ex)))
        "The call's own exception arrives as the ExecutionException's cause."))
  (let [gate (promise)
        fut  (sut/vfuture @gate)]
    (is (= ::timeout (deref fut 10 ::timeout)))
    (is (not (realized? fut)))
    (deliver gate :released)
    (is (= :released @fut)
        "An abandoned wait doesn't abandon the value; a later deref sees it.")
    (is (realized? fut))))

(deftest vfuture-cancel
  (let [started     (promise)
        interrupted (promise)
        fut         (sut/vfuture
                     (deliver started true)
                     (try @(promise)
                          (catch InterruptedException _
                            (deliver interrupted true))))]
    @started
    (is (true? (future-cancel fut)))
    (is (true? (deref interrupted 1000 ::not-interrupted))
        "Cancellation interrupts the virtual thread.")
    (is (future-cancelled? fut))
    (is (realized? fut))
    (is (thrown? CancellationException @fut)))
  (let [fut (sut/vfuture 1)]
    @fut
    (is (false? (future-cancel fut))
        "Cancelling a completed future is a no-op.")
    (is (= 1 @fut))))
