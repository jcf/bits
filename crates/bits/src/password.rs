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

    pub fn hash_password(
        &self,
        password: &bits_domain::Password,
    ) -> Result<bits_domain::PasswordHash, anyhow::Error> {
        use secrecy::ExposeSecret;
        let salt = SaltString::generate(&mut OsRng);
        self.argon2
            .hash_password(password.expose_secret().as_bytes(), &salt)
            .map(|hash| bits_domain::PasswordHash::new(hash.to_string()))
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))
    }

    pub fn verify_password(
        &self,
        password: &bits_domain::Password,
        hash: &bits_domain::PasswordHash,
    ) -> Result<(), anyhow::Error> {
        use secrecy::ExposeSecret;
        let parsed_hash = PasswordHash::new(hash.expose_secret())
            .map_err(|e| anyhow::anyhow!("Invalid password hash: {}", e))?;
        self.argon2
            .verify_password(password.expose_secret().as_bytes(), &parsed_hash)
            .map_err(|e| anyhow::anyhow!("Password verification failed: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> crate::Config {
        crate::Config {
            version: "test".to_string(),
            database_url: "postgresql://localhost/test".parse().unwrap(),
            max_database_connections: 5,
            argon2_m_cost: 19456, // 19 MiB for fast tests
            argon2_t_cost: 2,
            argon2_p_cost: 1,
            master_key: "test-key-32-bytes-long!!!!!!".to_string(),
            port: 0,
            session_name: "test".to_string(),
            platform_domain: "localhost".to_string(),
            dangerously_allow_javascript_evaluation: false,
            metrics_auth_token: None,
            global_rate_limit: Some(50),
        }
    }

    #[test]
    fn hash_password_produces_valid_hash() {
        use secrecy::ExposeSecret;

        let config = test_config();
        let service = PasswordService::new(&config).unwrap();
        let password = bits_domain::Password::new("test-password-123".to_string());

        let hash = service.hash_password(&password).unwrap();

        // Verify format: starts with $argon2id$
        assert!(hash.expose_secret().starts_with("$argon2id$v=19$"));
    }

    #[test]
    fn verify_password_succeeds_with_correct_password() {
        let config = test_config();
        let service = PasswordService::new(&config).unwrap();
        let password = bits_domain::Password::new("correct-password".to_string());

        let hash = service.hash_password(&password).unwrap();
        assert!(service.verify_password(&password, &hash).is_ok());
    }

    #[test]
    fn verify_password_fails_with_wrong_password() {
        let config = test_config();
        let service = PasswordService::new(&config).unwrap();
        let correct_password = bits_domain::Password::new("correct-password".to_string());
        let wrong_password = bits_domain::Password::new("wrong-password".to_string());

        let hash = service.hash_password(&correct_password).unwrap();
        assert!(service.verify_password(&wrong_password, &hash).is_err());
    }

    #[test]
    fn verify_password_fails_with_invalid_hash() {
        let config = test_config();
        let service = PasswordService::new(&config).unwrap();
        let password = bits_domain::Password::new("password".to_string());
        let invalid_hash = bits_domain::PasswordHash::new("not-a-hash".to_string());

        assert!(service.verify_password(&password, &invalid_hash).is_err());
    }

    #[test]
    fn hash_password_produces_unique_salts() {
        use secrecy::ExposeSecret;

        let config = test_config();
        let service = PasswordService::new(&config).unwrap();
        let password = bits_domain::Password::new("same-password".to_string());

        let hash1 = service.hash_password(&password).unwrap();
        let hash2 = service.hash_password(&password).unwrap();

        // Same password should produce different hashes due to random salt
        assert_ne!(hash1.expose_secret(), hash2.expose_secret());

        // But both should verify
        assert!(service.verify_password(&password, &hash1).is_ok());
        assert!(service.verify_password(&password, &hash2).is_ok());
    }
}
