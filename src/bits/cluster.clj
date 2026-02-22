(ns bits.cluster
  (:require
   [bits.crypto :as crypto]
   [bits.spec]
   [clojure.spec.alpha :as s]
   [clojure.string :as str]
   [com.stuartsierra.component :as component]
   [io.pedestal.log :as log]
   [java-time.api :as time]
   [steffan-westcott.clj-otel.api.trace.span :as span]
   [taoensso.nippy :as nippy])
  (:import
   (java.net InetAddress)
   (org.jgroups BytesMessage JChannel Receiver)
   (org.jgroups.protocols ASYM_ENCRYPT
                          BARRIER
                          FD_ALL3
                          FD_SOCK2
                          FRAG4
                          MERGE3
                          MFC
                          TCP
                          SSL_KEY_EXCHANGE
                          TCPPING
                          UFC
                          UNICAST3
                          VERIFY_SUSPECT)
   (org.jgroups.protocols.pbcast GMS
                                 NAKACK2
                                 STABLE)
   (org.jgroups.stack Protocol)))

;;; ----------------------------------------------------------------------------
;;; Peer name

(defn random-peer-name
  [peer]
  (str "bits-peer-"
       (-> (:randomizer peer)
           crypto/random-sid
           (str/replace #"[^a-z0-9]" "")
           (subs 0 6))))

;;; ----------------------------------------------------------------------------
;;; Stack

(defn- make-protocols
  [peer]
  (let [{:keys [bind-addr
                bind-port
                initial-hosts
                keystore-password
                keystore-path]} peer]
    (assert (some? keystore-password) "Missing keystore password?!")
    (assert (some? keystore-path) "Missing keystore path?!")
    (into-array Protocol
                [(doto (TCP.)
                   (.setValue "bind_addr" (InetAddress/getByName bind-addr))
                   (.setValue "bind_port" (int bind-port)))
                 (doto (TCPPING.)
                   (.setInitialHosts initial-hosts))
                 (MERGE3.)
                 (FD_SOCK2.)
                 (doto (FD_ALL3.)
                   (.setValue "interval" (int 2000))
                   (.setValue "timeout" (int 8000)))
                 (VERIFY_SUSPECT.)
                 (doto (SSL_KEY_EXCHANGE.)
                   (.setKeystoreName keystore-path)
                   (.setKeystorePassword keystore-password)
                   (.setKeystoreType "PKCS12")
                   (.setTruststoreName keystore-path)
                   (.setTruststorePassword keystore-password)
                   (.setTruststoreType "PKCS12"))
                 (BARRIER.)
                 (doto (ASYM_ENCRYPT.)
                   (.setUseExternalKeyExchange true))
                 (NAKACK2.)
                 (UNICAST3.)
                 (STABLE.)
                 (doto (GMS.)
                   (.setValue "print_local_addr" false))
                 (UFC.)
                 (MFC.)
                 (FRAG4.)])))

;;; ----------------------------------------------------------------------------
;;; Lifecycle

(defn- prepare
  "Creates peer state: name, view atom, protocol stack, and channel.
   Does not connect or attach a receiver."
  [peer]
  (let [peer-name (random-peer-name peer)
        view      (atom #{})
        protocols (make-protocols peer)
        chan       (-> (JChannel. protocols)
                       (.name peer-name))]
    {:chan       chan
     :peer-name peer-name
     :view      view}))

(defn view->map
  [^org.jgroups.View view]
  (let [members (.getMembers view)]
    {:coordinator (-> members first str)
     :id          (-> view .getViewId str)
     :members     (into #{} (map str) members)
     :size        (.size view)}))

(defn- attach-receiver
  "Attaches a JGroups Receiver to the peer's channel.
   Must be called before connect so the initial viewAccepted is captured."
  [peer handler]
  (let [{:keys [chan]} peer]
    (.setReceiver chan
                  (reify Receiver
                    (^void receive [_ ^org.jgroups.Message msg]
                      (span/with-span! {:name ::receive}
                        (try
                          (let [event (nippy/thaw (.getArray msg))]
                            (handler peer event))
                          (catch Exception ex
                            (log/warn :msg       "Error handling event?!"
                                      :peer      peer
                                      :exception ex))))
                      nil)
                    (^void viewAccepted [_ ^org.jgroups.View view]
                      (span/with-span! {:name ::view-accepted}
                        (reset! (:view peer) (view->map view)))
                      nil)))))

(defn- join
  "Connects the peer's channel to the cluster."
  [peer]
  (let [{:keys [chan cluster-name peer-name]} peer]
    (try
      (.connect chan cluster-name)
      (catch Exception ex
        (log/warn :msg          "Failed to join cluster?!"
                  :cluster-name cluster-name
                  :peer-name    peer-name
                  :exception    ex)))))

;;; ----------------------------------------------------------------------------
;;; Send

(defn event->bytes
  [peer event]
  (span/with-span! {:name ::event->bytes}
    (nippy/freeze
     (merge {:event/time (time/instant)
             :event/peer (:peer-name peer)}
            event))))

(defn send!
  [peer event]
  (span/with-span! {:name ::send!}
    (let [bytes (event->bytes peer event)]
      (.send (:chan peer) (BytesMessage. nil ^bytes bytes)))))

;;; ----------------------------------------------------------------------------
;;; Component

(defrecord Peer [bind-addr
                 bind-port
                 chan
                 cluster-name
                 initial-hosts
                 keystore-password
                 keystore-path
                 randomizer
                 view]
  component/Lifecycle
  (start [this]
    (span/with-span! {:name ::start}
      (let [peer (merge this (prepare this))]
        (attach-receiver peer (fn [_ event]
                                (log/info :msg   "Event received."
                                          :event event)))
        (span/with-span! {:name ::join}
          (join peer))
        peer)))
  (stop [this]
    (span/with-span! {:name ::stop}
      (when-let [ch (:chan this)]
        (.close ch))
      (assoc this :chan nil :view nil))))

(defmethod print-method Peer
  [_ ^java.io.Writer w]
  (.write w "#<Peer>"))

(s/fdef make-peer
  :args (s/cat :config ::config)
  :ret  ::config)

(defn make-peer
  [config]
  (map->Peer config))
