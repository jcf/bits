(ns bits.async
  (:import
   (java.util.concurrent CompletableFuture TimeUnit TimeoutException)))

(defn vfuture-call
  {:static true}
  [f]
  (let [f  (bound-fn* f)
        cf (CompletableFuture.)
        t  (Thread/startVirtualThread
            (fn [] (try (.complete cf (f))
                        (catch Throwable ex (.completeExceptionally cf ex)))))]
    (reify
      clojure.lang.IDeref
      (deref [_] (.get cf))

      clojure.lang.IBlockingDeref
      (deref [_ timeout-ms timeout-val]
        (try (.get cf timeout-ms TimeUnit/MILLISECONDS)
             (catch TimeoutException _ timeout-val)))

      clojure.lang.IPending
      (isRealized [_] (.isDone cf))

      java.util.concurrent.Future
      (get [_] (.get cf))
      (isCancelled [_] (.isCancelled cf))
      (isDone [_] (.isDone cf))
      (cancel [_ interrupt?]
        (let [cancelled (.cancel cf false)]
          (when (and cancelled interrupt?) (.interrupt t))
          cancelled)))))

(defmacro vfuture
  {:added "1.1"}
  [& body] `(vfuture-call (^{:once true} fn* [] ~@body)))

(comment
  (deref (vfuture (do (Thread/sleep 1000) ::done)) 500 ::timeout))
