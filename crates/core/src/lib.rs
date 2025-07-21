//! Core types and traits for Bits, please

use serde::{Serialize, Deserialize};
use thiserror::Error;

/// Unique identifier for network participants
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub [u8; 32]);

/// Decentralized identifier
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Did(pub String);

/// Resource contribution types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Contribution {
    Compute { cpu_hours: f64, gpu_hours: f64 },
    Storage { gigabyte_hours: f64 },
    Bandwidth { gigabytes_transferred: f64 },
    Capital { amount: u64, token_type: String },
    Development { commits: u32, lines_of_code: u32 },
}

/// Errors that can occur across the platform
#[derive(Error, Debug)]
pub enum PlatformError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Consensus error: {0}")]
    Consensus(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Identity error: {0}")]
    Identity(String),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Insufficient stake")]
    InsufficientStake,

    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<serde_json::Error> for PlatformError {
    fn from(err: serde_json::Error) -> Self {
        PlatformError::Serialization(err.to_string())
    }
}

/// Result type alias
pub type Result<T> = std::result::Result<T, PlatformError>;

/// Trait for components that can be initialized
#[async_trait::async_trait]
pub trait Component: Send + Sync {
    async fn start(&mut self) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;
}
