use anyhow::Result;
use libp2p::{
    core::upgrade,
    gossipsub, identity, kad, mdns, noise, tcp, yamux,
    swarm::{NetworkBehaviour, SwarmBuilder, SwarmEvent},
    PeerId, Transport,
};
use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}, time::Duration};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

pub mod discovery;
pub mod routing;
pub mod transport;

#[derive(NetworkBehaviour)]
pub struct BitsNetworkBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    pub mdns: mdns::tokio::Behaviour,
}

#[derive(Clone)]
pub struct Network {
    sender: mpsc::Sender<NetworkCommand>,
}

pub enum NetworkCommand {
    PublishContent { cid: String, metadata: Vec<u8> },
    FindContent { cid: String },
    StoreChunk { key: Vec<u8>, data: Vec<u8> },
    GetChunk { key: Vec<u8> },
}

impl Network {
    pub async fn new(port: u16, bootstrap_nodes: Vec<String>) -> Result<Self> {
        let (tx, mut rx) = mpsc::channel(100);

        // Generate keypair for this node
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        info!("Local peer id: {}", local_peer_id);

        // Create transport
        let transport = tcp::tokio::Transport::default()
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::Config::new(&local_key)?)
            .multiplex(yamux::Config::default())
            .boxed();

        // Configure gossipsub
        let message_id_fn = |message: &gossipsub::Message| {
            let mut hasher = DefaultHasher::new();
            message.data.hash(&mut hasher);
            gossipsub::MessageId::from(hasher.finish().to_string())
        };

        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(1))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .message_id_fn(message_id_fn)
            .build()
            .expect("Valid config");

        let mut gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
        )?;

        // Subscribe to content topic
        gossipsub.subscribe(&gossipsub::IdentTopic::new("bits/content/1.0.0"))?;

        // Configure Kademlia
        let mut kademlia = kad::Behaviour::new(
            local_peer_id,
            kad::store::MemoryStore::new(local_peer_id),
        );
        kademlia.set_mode(Some(kad::Mode::Server));

        // Configure mDNS for local discovery
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)?;

        // Create combined behaviour
        let behaviour = BitsNetworkBehaviour {
            gossipsub,
            kademlia,
            mdns,
        };

        // Build swarm
        let mut swarm = SwarmBuilder::with_tokio_executor(transport, behaviour, local_peer_id)
            .build();

        // Listen on all interfaces
        swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{}", port).parse()?)?;

        // Connect to bootstrap nodes
        for addr in bootstrap_nodes {
            match addr.parse() {
                Ok(multiaddr) => {
                    swarm.dial(multiaddr)?;
                    info!("Dialing bootstrap node: {}", addr);
                }
                Err(e) => warn!("Invalid bootstrap address {}: {}", addr, e),
            }
        }

        // Spawn network event loop
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    event = swarm.select_next_some() => {
                        handle_swarm_event(event).await;
                    }
                    Some(cmd) = rx.recv() => {
                        handle_network_command(&mut swarm, cmd).await;
                    }
                }
            }
        });

        Ok(Network { sender: tx })
    }

    pub async fn run(self) -> Result<()> {
        // Keep the network running
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    pub async fn publish_content(&self, cid: String, metadata: Vec<u8>) -> Result<()> {
        self.sender
            .send(NetworkCommand::PublishContent { cid, metadata })
            .await?;
        Ok(())
    }

    pub async fn find_content(&self, cid: String) -> Result<()> {
        self.sender
            .send(NetworkCommand::FindContent { cid })
            .await?;
        Ok(())
    }
}

async fn handle_swarm_event(event: SwarmEvent<BitsNetworkBehaviour>) {
    match event {
        SwarmEvent::NewListenAddr { address, .. } => {
            info!("Listening on {}", address);
        }
        SwarmEvent::Behaviour(BitsNetworkBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
            for (peer_id, _multiaddr) in list {
                debug!("mDNS discovered peer: {}", peer_id);
            }
        }
        SwarmEvent::Behaviour(BitsNetworkBehaviourEvent::Kademlia(kad::Event::RoutingUpdated { peer, .. })) => {
            debug!("Routing updated for peer: {}", peer);
        }
        _ => {}
    }
}

async fn handle_network_command(
    swarm: &mut libp2p::Swarm<BitsNetworkBehaviour>,
    command: NetworkCommand,
) {
    match command {
        NetworkCommand::PublishContent { cid, metadata } => {
            let topic = gossipsub::IdentTopic::new("bits/content/1.0.0");
            if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic, metadata) {
                warn!("Failed to publish content {}: {}", cid, e);
            } else {
                info!("Published content: {}", cid);
            }
        }
        NetworkCommand::StoreChunk { key, data } => {
            let record = kad::Record {
                key: kad::RecordKey::new(&key),
                value: data,
                publisher: None,
                expires: None,
            };
            let _ = swarm.behaviour_mut().kademlia.put_record(record, kad::Quorum::One);
            debug!("Stored chunk with key: {:?}", key);
        }
        _ => {}
    }
}