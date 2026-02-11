(ns bits.test.fixture
  (:require [java-time.api :as time]))

(defn realm-txes
  ([] (realm-txes {}))
  ([overrides]
   (let [defaults   {:creator/display-name "Test"
                     :creator/handle       "test"
                     :domain/name          "localhost"
                     :tenant/created-at    (time/java-date)
                     :tenant/id            (random-uuid)}
         attributes (merge defaults overrides)]
     [{:db/id       "domain"
       :domain/name (:domain/name attributes)}
      (-> attributes
          (dissoc :domain/name)
          (assoc :tenant/domains ["domain"]))])))
