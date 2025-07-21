use crate::{Identity, IdentityKeys, KeyPair, Did, DidDocument, DidExt};
use bits_core::{PlatformError, Result};
use serde::{Serialize, Deserialize};
use sled::Db;
use tracing::info;

const IDENTITY_KEY: &str = "primary_identity";

/// Persistent storage for identities
pub struct IdentityStore {
    db: Db,
}

/// Stored identity data (encrypted)
#[derive(Serialize, Deserialize)]
pub struct StoredIdentity {
    pub did: String,
    pub document: Vec<u8>, // Encrypted DID document
    pub keys: Vec<u8>,     // Encrypted keys
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub salt: Vec<u8>,
}

/// Encrypted key material
#[derive(Serialize, Deserialize)]
struct EncryptedKeys {
    master: Vec<u8>,
    signing: Vec<u8>,
    encryption: Vec<u8>,
    authentication: Vec<u8>,
    nonce: Vec<u8>,
}

impl IdentityStore {
    /// Create a new identity store
    pub fn new(path: &str) -> Result<Self> {
        let db = sled::open(path)
            .map_err(|e| PlatformError::Storage(format!("Failed to open database: {}", e)))?;
        
        Ok(Self { db })
    }
    
    /// Save an identity to storage
    pub async fn save_identity(&self, identity: &Identity) -> Result<()> {
        // Generate salt for key derivation
        let mut salt = vec![0u8; 32];
        use rand::RngCore;
        rand::thread_rng().fill_bytes(&mut salt);
        
        // For now, store unencrypted (in production, use device key)
        let stored = StoredIdentity {
            did: identity.did.to_string(),
            document: serde_json::to_vec(&identity.document)?,
            keys: self.serialize_keys(&identity.keys)?,
            created_at: identity.created_at,
            salt,
        };
        
        // Store as primary identity
        let key = IDENTITY_KEY;
        let value = serde_json::to_vec(&stored)?;
        
        self.db.insert(key, value)
            .map_err(|e| PlatformError::Storage(format!("Failed to save identity: {}", e)))?;
        
        self.db.flush()
            .map_err(|e| PlatformError::Storage(format!("Failed to flush: {}", e)))?;
        
        info!("Saved identity: {}", identity.did);
        Ok(())
    }
    
    /// Load the primary identity
    pub async fn load_primary(&self) -> Result<Option<StoredIdentity>> {
        match self.db.get(IDENTITY_KEY) {
            Ok(Some(data)) => {
                let stored: StoredIdentity = serde_json::from_slice(&data)?;
                Ok(Some(stored))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(PlatformError::Storage(format!("Failed to load identity: {}", e))),
        }
    }
    
    /// Export identity with password encryption
    pub async fn export_encrypted(&self, identity: &Identity, password: &str) -> Result<Vec<u8>> {
        use chacha20poly1305::{
            aead::{Aead, AeadCore, KeyInit, OsRng},
            ChaCha20Poly1305,
        };
        
        // Generate salt
        let mut salt = [0u8; 32];
        use rand::RngCore;
        OsRng.fill_bytes(&mut salt);
        
        // Derive key from password
        let key = crate::keys::derive_key_from_password(password, &salt);
        
        // Create cipher
        let cipher = ChaCha20Poly1305::new_from_slice(&key)
            .map_err(|e| PlatformError::Crypto(format!("Cipher error: {}", e)))?;
        
        // Generate nonce
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        
        // Serialize identity
        let identity_data = serde_json::to_vec(&(
            &identity.did,
            &identity.document,
            self.serialize_keys(&identity.keys)?,
            &identity.created_at,
        ))?;
        
        // Encrypt
        let ciphertext = cipher.encrypt(&nonce, identity_data.as_ref())
            .map_err(|e| PlatformError::Crypto(format!("Encryption failed: {}", e)))?;
        
        // Package: salt + nonce + ciphertext
        let mut export = Vec::new();
        export.extend_from_slice(&salt);
        export.extend_from_slice(nonce.as_ref());
        export.extend_from_slice(&ciphertext);
        
        Ok(export)
    }
    
    /// Import identity from encrypted backup
    pub async fn import_encrypted(&self, backup: &[u8], password: &str) -> Result<Identity> {
        use chacha20poly1305::{
            aead::{Aead, KeyInit},
            ChaCha20Poly1305, Nonce,
        };
        
        if backup.len() < 32 + 12 {
            return Err(PlatformError::Crypto("Invalid backup format".into()));
        }
        
        // Extract components
        let salt = &backup[..32];
        let nonce = &backup[32..44];
        let ciphertext = &backup[44..];
        
        // Derive key
        let key = crate::keys::derive_key_from_password(password, salt);
        
        // Create cipher
        let cipher = ChaCha20Poly1305::new_from_slice(&key)
            .map_err(|e| PlatformError::Crypto(format!("Cipher error: {}", e)))?;
        
        // Decrypt
        let nonce = Nonce::from_slice(nonce);
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| PlatformError::Crypto(format!("Decryption failed: {}", e)))?;
        
        // Deserialize
        let (did_str, document, keys_data, created_at): (String, DidDocument, Vec<u8>, chrono::DateTime<chrono::Utc>) = 
            serde_json::from_slice(&plaintext)?;
        
        let did = Did::parse(&did_str).map_err(|e| PlatformError::Identity(e.to_string()))?;
        let keys = self.deserialize_keys(&keys_data)?;
        
        Ok(Identity {
            did,
            document,
            keys,
            created_at,
        })
    }
    
    /// Flush any pending writes
    pub async fn flush(&self) -> Result<()> {
        self.db.flush()
            .map_err(|e| PlatformError::Storage(format!("Failed to flush: {}", e)))?;
        Ok(())
    }
    
    /// Serialize identity keys
    fn serialize_keys(&self, keys: &IdentityKeys) -> Result<Vec<u8>> {
        // In production, these would be encrypted with device key
        let data = (
            keys.master.export_secret(),
            keys.signing.export_secret(),
            keys.encryption,
            keys.authentication.export_secret(),
        );
        
        Ok(serde_json::to_vec(&data)?)
    }
    
    /// Deserialize identity keys
    fn deserialize_keys(&self, data: &[u8]) -> Result<IdentityKeys> {
        let (master, signing, encryption, auth): ([u8; 32], [u8; 32], [u8; 32], [u8; 32]) = 
            serde_json::from_slice(data)?;
        
        Ok(IdentityKeys {
            master: KeyPair::from_secret_bytes(&master)
                .map_err(|e| PlatformError::Crypto(e.to_string()))?,
            signing: KeyPair::from_secret_bytes(&signing)
                .map_err(|e| PlatformError::Crypto(e.to_string()))?,
            encryption,
            authentication: KeyPair::from_secret_bytes(&auth)
                .map_err(|e| PlatformError::Crypto(e.to_string()))?,
        })
    }
}

impl StoredIdentity {
    /// Decrypt stored identity (simplified for now)
    pub fn decrypt(self) -> Result<Identity> {
        // In production, decrypt with device key
        let document: DidDocument = serde_json::from_slice(&self.document)?;
        let did = Did::parse(&self.did).map_err(|e| PlatformError::Identity(e.to_string()))?;
        
        // For now, keys are stored in a simple format
        // In production, these would be encrypted
        let keys = serde_json::from_slice(&self.keys)?;
        
        Ok(Identity {
            did,
            document,
            keys,
            created_at: self.created_at,
        })
    }
}

impl Serialize for IdentityKeys {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("IdentityKeys", 4)?;
        state.serialize_field("master", &self.master.export_secret())?;
        state.serialize_field("signing", &self.signing.export_secret())?;
        state.serialize_field("encryption", &self.encryption)?;
        state.serialize_field("authentication", &self.authentication.export_secret())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for IdentityKeys {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Keys {
            master: [u8; 32],
            signing: [u8; 32],
            encryption: [u8; 32],
            authentication: [u8; 32],
        }
        
        let keys = Keys::deserialize(deserializer)?;
        
        Ok(IdentityKeys {
            master: KeyPair::from_secret_bytes(&keys.master)
                .map_err(serde::de::Error::custom)?,
            signing: KeyPair::from_secret_bytes(&keys.signing)
                .map_err(serde::de::Error::custom)?,
            encryption: keys.encryption,
            authentication: KeyPair::from_secret_bytes(&keys.authentication)
                .map_err(serde::de::Error::custom)?,
        })
    }
}