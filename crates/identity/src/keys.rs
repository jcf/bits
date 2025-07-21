use ed25519_dalek::{Signature, Signer, SigningKey as DalekSigningKey, VerifyingKey, Verifier};
use serde::{Deserialize, Serialize};

/// A key pair for signing operations
#[derive(Clone)]
pub struct KeyPair {
    pub public: PublicKey,
    secret: SecretKey,
}

/// Public key wrapper
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicKey(VerifyingKey);

/// Secret key wrapper with automatic zeroing
#[derive(Clone)]
struct SecretKey(DalekSigningKey);

impl KeyPair {
    /// Generate a new random keypair
    pub fn generate() -> Self {
        let secret = DalekSigningKey::generate(&mut rand::thread_rng());
        let public = secret.verifying_key();

        Self {
            public: PublicKey(public),
            secret: SecretKey(secret),
        }
    }

    /// Create from raw secret bytes (32 bytes)
    pub fn from_secret_bytes(bytes: &[u8; 32]) -> Result<Self, KeyError> {
        let secret = DalekSigningKey::from_bytes(bytes);
        let public = secret.verifying_key();

        Ok(Self {
            public: PublicKey(public),
            secret: SecretKey(secret),
        })
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        let signature = self.secret.0.sign(message);
        signature.to_bytes().to_vec()
    }

    /// Export secret key (careful with this!)
    pub fn export_secret(&self) -> [u8; 32] {
        self.secret.0.to_bytes()
    }

    /// Derive a child key using HKDF
    pub fn derive_child(&self, info: &[u8]) -> Self {
        use blake3::Hasher;

        // Use Blake3 for key derivation (simpler than HKDF for our use case)
        let mut hasher = Hasher::new_derive_key("bits-identity-derive");
        hasher.update(&self.secret.0.to_bytes());
        hasher.update(info);

        let mut derived = [0u8; 32];
        hasher.finalize_xof().fill(&mut derived);

        Self::from_secret_bytes(&derived).expect("Valid key material")
    }
}

impl PublicKey {
    /// Create from raw bytes (32 bytes)
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, KeyError> {
        let key = VerifyingKey::from_bytes(bytes).map_err(|_| KeyError::InvalidPublicKey)?;
        Ok(PublicKey(key))
    }

    /// Get raw bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        self.0.as_bytes()
    }

    /// Verify a signature
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> bool {
        if let Ok(sig) = Signature::from_slice(signature) {
            self.0.verify(message, &sig).is_ok()
        } else {
            false
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum KeyError {
    #[error("Invalid public key")]
    InvalidPublicKey,
    #[error("Invalid secret key")]
    InvalidSecretKey,
    #[error("Signature verification failed")]
    VerificationFailed,
}

/// Secure key derivation from password
pub fn derive_key_from_password(password: &str, salt: &[u8]) -> [u8; 32] {
    use argon2::password_hash::SaltString;
    use argon2::{Argon2, PasswordHasher};

    // Use Argon2id with recommended parameters
    let argon2 = Argon2::default();

    // Create salt string
    let salt_str = SaltString::encode_b64(salt).unwrap();

    // Hash password
    let hash = argon2
        .hash_password(password.as_bytes(), &salt_str)
        .unwrap();

    // Extract 32 bytes for key
    let hash_bytes = hash.hash.unwrap();
    let mut key = [0u8; 32];
    key.copy_from_slice(&hash_bytes.as_bytes()[..32]);

    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let kp = KeyPair::generate();
        let message = b"test message";
        let signature = kp.sign(message);

        assert!(kp.public.verify(message, &signature));
        assert!(!kp.public.verify(b"wrong message", &signature));
    }

    #[test]
    fn test_key_derivation() {
        let parent = KeyPair::generate();
        let child1 = parent.derive_child(b"child1");
        let child2 = parent.derive_child(b"child2");

        // Children should be different
        assert_ne!(child1.public.as_bytes(), child2.public.as_bytes());

        // Same info should produce same child
        let child1_again = parent.derive_child(b"child1");
        assert_eq!(child1.public.as_bytes(), child1_again.public.as_bytes());
    }
}

impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(self.as_bytes())
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes: [u8; 32] = Deserialize::deserialize(deserializer)?;
        PublicKey::from_bytes(&bytes)
            .map_err(serde::de::Error::custom)
    }
}
