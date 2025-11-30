(ns bits.router)

(defn combine
  [& routes]
  (reduce (fn [t x] (cond-> t (some? x) (into x))) #{} routes))

(comment
  (combine
   [["/a" :get (constantly "a")] ["/b" :get (constantly "b")]]
   []
   [["/c" :get (constantly "c")]]
   nil))
