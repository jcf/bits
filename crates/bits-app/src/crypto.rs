//! Encryption and HMAC service using Orion
//!
//! This module provides centralized cryptographic operations including:
//! - Key derivation using HKDF
//! - HMAC signing and verification for tokens
//! - Secure key management from master key

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use orion::hazardous::kdf::hkdf;
use orion::hazardous::mac::hmac::sha512::{HmacSha512, SecretKey, Tag};
use std::sync::Arc;

/// Encryption service for HMAC signing and key derivation
///
/// Uses Arc internally to make it cheaply cloneable despite SecretKey
/// not implementing Clone.
#[derive(Clone)]
pub struct EncryptionService {
    master_key: Arc<SecretKey>,
}

impl EncryptionService {
    /// Create a new encryption service from a base64-encoded master key
    ///
    /// The master key must be at least 64 bytes when decoded (for HMAC-SHA512).
    pub fn new(master_key_base64: &str) -> Result<Self, EncryptionServiceError> {
        let decoded = BASE64
            .decode(master_key_base64)
            .map_err(|e| EncryptionServiceError::InvalidMasterKey(e.to_string()))?;

        if decoded.len() < 64 {
            return Err(EncryptionServiceError::InvalidMasterKey(
                "Master key must be at least 64 bytes for HMAC-SHA512".to_string(),
            ));
        }

        let master_key = SecretKey::from_slice(&decoded)
            .map_err(|e| EncryptionServiceError::InvalidMasterKey(e.to_string()))?;

        Ok(Self {
            master_key: Arc::new(master_key),
        })
    }

    /// Derive a key for a specific purpose using HKDF
    ///
    /// This allows deriving different keys for different purposes from the master key.
    /// For example: "email-verification", "password-reset", "data-encryption"
    pub fn derive_key(&self, purpose: &str, salt: &[u8]) -> Result<Vec<u8>, EncryptionServiceError> {
        let mut output = [0u8; 32];
        let info = Some(purpose.as_bytes());

        hkdf::sha512::derive_key(
            salt,
            self.master_key.unprotected_as_bytes(),
            info,
            &mut output,
        )
        .map_err(|e| EncryptionServiceError::KeyDerivation(e.to_string()))?;

        Ok(output.to_vec())
    }

    /// Generate HMAC signature for data
    ///
    /// Uses the master key to sign arbitrary data. This is useful for
    /// creating tamper-proof tokens.
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, EncryptionServiceError> {
        let mut state = HmacSha512::new(&self.master_key);
        state
            .update(data)
            .map_err(|e| EncryptionServiceError::Signing(e.to_string()))?;
        let tag = state
            .finalize()
            .map_err(|e| EncryptionServiceError::Signing(e.to_string()))?;

        Ok(tag.unprotected_as_bytes().to_vec())
    }

    /// Verify HMAC signature for data
    ///
    /// Returns Ok(()) if the signature is valid, Err otherwise.
    pub fn verify(&self, signature: &[u8], data: &[u8]) -> Result<(), EncryptionServiceError> {
        let tag = Tag::from_slice(signature)
            .map_err(|e| EncryptionServiceError::Verification(e.to_string()))?;

        HmacSha512::verify(&tag, &self.master_key, data)
            .map_err(|_| EncryptionServiceError::Verification("Invalid signature".to_string()))
    }

    /// Sign data and encode as base64
    ///
    /// Convenience method that signs data and returns base64-encoded signature.
    pub fn sign_base64(&self, data: &[u8]) -> Result<String, EncryptionServiceError> {
        let signature = self.sign(data)?;
        Ok(BASE64.encode(&signature))
    }

    /// Verify base64-encoded signature
    ///
    /// Convenience method that decodes base64 signature and verifies it.
    pub fn verify_base64(&self, signature_base64: &str, data: &[u8]) -> Result<(), EncryptionServiceError> {
        let signature = BASE64
            .decode(signature_base64)
            .map_err(|e| EncryptionServiceError::Verification(e.to_string()))?;

        self.verify(&signature, data)
    }
}

impl std::fmt::Debug for EncryptionService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EncryptionService")
            .field("master_key", &"<redacted>")
            .finish()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EncryptionServiceError {
    #[error("Invalid master key: {0}")]
    InvalidMasterKey(String),
    #[error("Key derivation failed: {0}")]
    KeyDerivation(String),
    #[error("Signing failed: {0}")]
    Signing(String),
    #[error("Verification failed: {0}")]
    Verification(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_master_key() -> String {
        // Test key: 64 bytes of random data, base64 encoded (for HMAC-SHA512)
        BASE64.encode(&[42u8; 64])
    }

    #[test]
    fn test_new_encryption_service() {
        let service = EncryptionService::new(&test_master_key());
        assert!(service.is_ok());
    }

    #[test]
    fn test_invalid_base64() {
        let result = EncryptionService::new("not-valid-base64!!!");
        assert!(result.is_err());
    }

    #[test]
    fn test_key_too_short() {
        let short_key = BASE64.encode(&[42u8; 32]); // Only 32 bytes (need 64)
        let result = EncryptionService::new(&short_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_sign_and_verify() {
        let service = EncryptionService::new(&test_master_key()).unwrap();
        let data = b"test message";

        let signature = service.sign(data).unwrap();
        let result = service.verify(&signature, data);

        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_invalid_signature() {
        let service = EncryptionService::new(&test_master_key()).unwrap();
        let data = b"test message";

        let wrong_signature = vec![0u8; 64];
        let result = service.verify(&wrong_signature, data);

        assert!(result.is_err());
    }

    #[test]
    fn test_verify_wrong_data() {
        let service = EncryptionService::new(&test_master_key()).unwrap();
        let data = b"test message";

        let signature = service.sign(data).unwrap();
        let wrong_data = b"wrong message";
        let result = service.verify(&signature, wrong_data);

        assert!(result.is_err());
    }

    #[test]
    fn test_sign_verify_base64() {
        let service = EncryptionService::new(&test_master_key()).unwrap();
        let data = b"test message";

        let signature = service.sign_base64(data).unwrap();
        let result = service.verify_base64(&signature, data);

        assert!(result.is_ok());
    }

    #[test]
    fn test_derive_key() {
        let service = EncryptionService::new(&test_master_key()).unwrap();
        let salt = b"test-salt";

        let key1 = service.derive_key("email-verification", salt).unwrap();
        let key2 = service.derive_key("email-verification", salt).unwrap();
        let key3 = service.derive_key("password-reset", salt).unwrap();

        // Same purpose and salt should produce same key
        assert_eq!(key1, key2);

        // Different purpose should produce different key
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_derive_key_different_salts() {
        let service = EncryptionService::new(&test_master_key()).unwrap();

        let key1 = service.derive_key("test", b"salt1").unwrap();
        let key2 = service.derive_key("test", b"salt2").unwrap();

        // Different salts should produce different keys
        assert_ne!(key1, key2);
    }
}
