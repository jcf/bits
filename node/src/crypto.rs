use anyhow::Result;
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305,
};

pub type Key = chacha20poly1305::Key;
pub type Nonce = chacha20poly1305::Nonce;

/// Generate a new encryption key
pub fn generate_key() -> Key {
    ChaCha20Poly1305::generate_key(&mut OsRng)
}

/// Generate a new nonce
pub fn generate_nonce() -> Nonce {
    ChaCha20Poly1305::generate_nonce(&mut OsRng)
}

/// Encrypt data with ChaCha20-Poly1305
pub fn encrypt(key: &Key, nonce: &Nonce, plaintext: &[u8]) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(key);
    cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))
}

/// Decrypt data with ChaCha20-Poly1305
pub fn decrypt(key: &Key, nonce: &Nonce, ciphertext: &[u8]) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(key);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))
}

/// Derive a key from a password using Argon2
pub fn derive_key(password: &[u8], salt: &[u8]) -> Result<Key> {
    use argon2::{Algorithm, Argon2, Params, Version};

    let mut key = Key::default();

    let argon2 = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2, 1, Some(key.len())).map_err(|e| anyhow::anyhow!("{}", e))?,
    );

    argon2
        .hash_password_into(password, salt, &mut key)
        .map_err(|e| anyhow::anyhow!("Key derivation failed: {:?}", e))?;

    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let key1 = generate_key();
        let key2 = generate_key();

        // Keys should be different
        assert_ne!(key1, key2);

        // Keys should be correct length
        assert_eq!(key1.len(), 32);
    }

    #[test]
    fn test_nonce_generation() {
        let nonce1 = generate_nonce();
        let nonce2 = generate_nonce();

        // Nonces should be different
        assert_ne!(nonce1, nonce2);

        // Nonces should be correct length
        assert_eq!(nonce1.len(), 12);
    }

    #[test]
    fn test_encrypt_decrypt() {
        let key = generate_key();
        let nonce = generate_nonce();
        let plaintext = b"Hello, Bits!";

        // Encrypt
        let ciphertext = encrypt(&key, &nonce, plaintext).unwrap();
        assert_ne!(ciphertext, plaintext);

        // Decrypt
        let decrypted = decrypt(&key, &nonce, &ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_with_wrong_key_fails() {
        let key = generate_key();
        let wrong_key = generate_key();
        let nonce = generate_nonce();
        let plaintext = b"Secret message";

        let ciphertext = encrypt(&key, &nonce, plaintext).unwrap();

        // Decryption with wrong key should fail
        assert!(decrypt(&wrong_key, &nonce, &ciphertext).is_err());
    }

    #[test]
    fn test_key_derivation() {
        let password = b"strong_password";
        let salt = b"random_salt_16_b";

        let key1 = derive_key(password, salt).unwrap();
        let key2 = derive_key(password, salt).unwrap();

        // Same password and salt should produce same key
        assert_eq!(key1, key2);

        // Different salt should produce different key
        let different_salt = b"different_salt16";
        let key3 = derive_key(password, different_salt).unwrap();
        assert_ne!(key1, key3);
    }
}
