//! Password hashing and verification using Argon2id.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

#[derive(Clone)]
pub struct PasswordService {
    argon2: Argon2<'static>,
}

impl PasswordService {
    pub fn new(config: &crate::Config) -> Result<Self, anyhow::Error> {
        let params = argon2::Params::new(
            config.argon2_m_cost,
            config.argon2_t_cost,
            config.argon2_p_cost,
            Some(argon2::Params::DEFAULT_OUTPUT_LEN),
        )
        .map_err(|e| anyhow::anyhow!("Invalid Argon2 parameters: {}", e))?;

        let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

        Ok(Self { argon2 })
    }

    pub fn hash_password(&self, password: &str) -> Result<String, anyhow::Error> {
        let salt = SaltString::generate(&mut OsRng);
        self.argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> Result<(), anyhow::Error> {
        let parsed_hash =
            PasswordHash::new(hash).map_err(|e| anyhow::anyhow!("Invalid password hash: {}", e))?;
        self.argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|e| anyhow::anyhow!("Password verification failed: {}", e))
    }
}
