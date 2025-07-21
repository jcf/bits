#![doc = include_str!("../README.md")]

mod did;
mod keys;
mod store;
mod username;

pub use did::{Did, DidDocument, DidMethod};
pub use keys::{KeyPair, PublicKey};
pub use store::{IdentityStore, StoredIdentity};
pub use username::{generate_username, parse_username};

use async_trait::async_trait;
use bits_core::{Component, PlatformError, Result};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Main identity service managing all identity operations
pub struct IdentityService {
    store: Arc<IdentityStore>,
    identity: Arc<RwLock<Option<Identity>>>,
}

/// Complete identity including keys and metadata
#[derive(Clone)]
pub struct Identity {
    pub did: Did,
    pub document: DidDocument,
    pub keys: IdentityKeys,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// All keys associated with an identity
#[derive(Clone)]
pub struct IdentityKeys {
    pub master: KeyPair,
    pub signing: KeyPair,
    pub encryption: [u8; 32], // Store raw bytes for simplicity
    pub authentication: KeyPair,
}

impl IdentityService {
    /// Create a new identity service with the given storage path
    pub async fn new(storage_path: &str) -> Result<Self> {
        let store = Arc::new(IdentityStore::new(storage_path)?);

        Ok(Self {
            store: store.clone(),
            identity: Arc::new(RwLock::new(None)),
        })
    }

    /// Load existing identity or create a new one
    pub async fn load_or_create(&self) -> Result<Identity> {
        // Check if we already have an identity loaded
        if let Some(identity) = self.identity.read().await.as_ref() {
            return Ok(identity.clone());
        }

        // Try to load from store
        let mut identity_lock = self.identity.write().await;

        match self.store.load_primary().await? {
            Some(stored) => {
                info!("Loaded existing identity: {}", stored.did);
                let identity = self.restore_from_stored(stored).await?;
                *identity_lock = Some(identity.clone());
                Ok(identity)
            }
            None => {
                info!("Creating new identity");
                let identity = self.create_new_identity().await?;
                *identity_lock = Some(identity.clone());
                Ok(identity)
            }
        }
    }

    /// Create a brand new identity
    async fn create_new_identity(&self) -> Result<Identity> {
        // Generate keys
        let keys = IdentityKeys::generate();

        // Create DID
        let did = Did::from_key(&keys.master.public);

        // Build DID document
        let document = DidDocument::new(did.clone())
            .with_verification_method(keys.master.public.clone(), "master")
            .with_verification_method(keys.signing.public.clone(), "signing")
            .with_verification_method(keys.authentication.public.clone(), "auth")
            .with_authentication("auth")
            .with_assertion_method("signing");

        let identity = Identity {
            did: did.clone(),
            document,
            keys,
            created_at: chrono::Utc::now(),
        };

        // Store it
        self.store.save_identity(&identity).await?;

        info!("Created new identity: {}", did);

        Ok(identity)
    }

    /// Restore identity from stored data
    async fn restore_from_stored(&self, stored: StoredIdentity) -> Result<Identity> {
        stored.decrypt()
    }

    /// Get the current identity
    pub async fn current(&self) -> Result<Identity> {
        self.identity.read().await
                            .as_ref()
                            .cloned()
                            .ok_or_else(|| PlatformError::Identity("No identity loaded".into()))
    }

    /// Sign a message with the signing key
    pub async fn sign(&self, message: &[u8]) -> Result<Vec<u8>> {
        let identity = self.current().await?;
        Ok(identity.keys.signing.sign(message))
    }

    /// Export identity for backup
    pub async fn export_backup(&self, password: &str) -> Result<Vec<u8>> {
        let identity = self.current().await?;
        self.store.export_encrypted(&identity, password).await
    }

    /// Import identity from backup
    pub async fn import_backup(&self, backup: &[u8], password: &str) -> Result<()> {
        let identity = self.store.import_encrypted(backup, password).await?;
        *self.identity.write().await = Some(identity);
        Ok(())
    }
}

#[async_trait]
impl Component for IdentityService {
    async fn start(&mut self) -> Result<()> {
        info!("Starting identity service");
        // Load identity on startup
        self.load_or_create().await?;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        info!("Stopping identity service");
        // Ensure any pending writes are flushed
        self.store.flush().await?;
        Ok(())
    }
}

impl IdentityKeys {
    /// Generate a full set of identity keys
    pub fn generate() -> Self {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut encryption = [0u8; 32];
        rng.fill_bytes(&mut encryption);

        Self {
            master: KeyPair::generate(),
            signing: KeyPair::generate(),
            encryption,
            authentication: KeyPair::generate(),
        }
    }
}
