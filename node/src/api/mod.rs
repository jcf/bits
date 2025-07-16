use crate::{blockchain, p2p, storage};
use anyhow::Result;
use std::sync::Arc;
use tracing::info;

#[derive(Clone)]
pub struct Server {
    port: u16,
    p2p: p2p::Network,
    storage: storage::Storage,
    blockchain: blockchain::Client,
}

impl Server {
    pub fn new(
        port: u16,
        p2p: p2p::Network,
        storage: storage::Storage,
        blockchain: blockchain::Client,
    ) -> Self {
        Server {
            port,
            p2p,
            storage,
            blockchain,
        }
    }

    pub async fn run(self) -> Result<()> {
        info!("API server starting on port {}", self.port);
        
        // TODO: Implement HTTP/WebSocket API
        // - Upload content
        // - Download content
        // - List available content
        // - Node statistics
        
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    }
}