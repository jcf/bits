use crate::{blockchain, p2p, storage};
use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

#[derive(Clone)]
pub struct Server {
    port: u16,
    p2p: p2p::Network,
    storage: storage::Storage,
    blockchain: blockchain::Client,
}

#[derive(Clone)]
struct AppState {
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

        let state = Arc::new(AppState {
            p2p: self.p2p.clone(),
            storage: self.storage.clone(),
            blockchain: self.blockchain.clone(),
        });

        let app = Router::new()
            .route("/health", get(health))
            .route("/stats", get(get_stats))
            .route("/content", post(publish_content))
            .route("/content/:cid", get(get_content))
            .route("/peers", get(get_peers))
            .with_state(state);

        let addr = format!("0.0.0.0:{}", self.port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        info!("API server listening on {}", addr);

        axum::serve(listener, app).await?;

        Ok(())
    }
}

async fn health() -> &'static str {
    "OK"
}

#[derive(Serialize)]
struct Stats {
    version: &'static str,
    network: &'static str,
    peers: usize,
    storage_stats: storage::StorageStats,
}

async fn get_stats(State(state): State<Arc<AppState>>) -> Result<Json<Stats>, StatusCode> {
    let storage_stats = state
        .storage
        .get_storage_stats()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(Stats {
        version: env!("CARGO_PKG_VERSION"),
        network: "mainnet",
        peers: 0, // TODO: Get from p2p network
        storage_stats,
    }))
}

#[derive(Deserialize)]
struct PublishRequest {
    creator: String,
    size: i64,
    price: i64,
    encrypted_key: Vec<u8>,
    chunks: Vec<ChunkData>,
}

#[derive(Deserialize)]
struct ChunkData {
    key: Vec<u8>,
    data: Vec<u8>,
}

async fn publish_content(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PublishRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Store chunks
    for chunk in req.chunks {
        state
            .storage
            .store_chunk(&chunk.key, &chunk.data)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Generate CID (simplified for now)
    let cid = format!("Qm{}", hex::encode(&req.encrypted_key[..16]));

    // Store metadata
    state
        .storage
        .store_content_metadata(&cid, &req.creator, req.size, req.price, &req.encrypted_key)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Publish to P2P network
    state
        .p2p
        .publish_content(cid.clone(), req.encrypted_key)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "cid": cid })))
}

async fn get_content(
    State(state): State<Arc<AppState>>,
    Path(cid): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Find content in P2P network
    state
        .p2p
        .find_content(cid.clone())
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(serde_json::json!({ "cid": cid, "status": "found" })))
}

async fn get_peers(State(_state): State<Arc<AppState>>) -> Json<Vec<String>> {
    // TODO: Get actual peers from P2P network
    Json(vec![])
}
