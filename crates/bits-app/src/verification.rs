//! Email verification service with 6-digit codes
//!
//! Provides secure email verification using HMAC-SHA256-hashed 6-digit codes with:
//! - Configurable expiration times
//! - Rate limiting for resends
//! - Attempt tracking to prevent brute force
//! - Same code on resend (prevents DoS)
//! - HKDF-derived keys from master secret

use ipnetwork::IpNetwork;
use jiff::{Span, Timestamp};
use sqlx::{types::chrono::DateTime, PgPool};
use std::net::IpAddr;

#[derive(Clone, Debug)]
pub struct EmailVerificationConfig {
    pub code_expiry_hours: i64,
    pub max_verification_attempts: i32,
    pub resend_cooldown_secs: i64,
    pub max_resends_per_hour: i64,
}

impl Default for EmailVerificationConfig {
    fn default() -> Self {
        Self {
            code_expiry_hours: 1,
            max_verification_attempts: 3,
            resend_cooldown_secs: 60,
            max_resends_per_hour: 5,
        }
    }
}

#[derive(Clone)]
pub struct EmailVerificationService {
    config: EmailVerificationConfig,
    hmac_secret: Vec<u8>,
}

impl EmailVerificationService {
    pub fn new(config: EmailVerificationConfig, hmac_secret: Vec<u8>) -> Self {
        Self {
            config,
            hmac_secret,
        }
    }

    pub fn with_config(mut self, config: EmailVerificationConfig) -> Self {
        self.config = config;
        self
    }

    /// Generate a random 6-digit code
    fn generate_code() -> String {
        use rand::Rng;
        let mut rng = rand::rng();
        format!("{:06}", rng.random_range(0..1000000))
    }

    /// Hash a code using HMAC-SHA256 for secure storage
    fn hash_code(&self, code: &str) -> String {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        let mut mac =
            Hmac::<Sha256>::new_from_slice(&self.hmac_secret).expect("HMAC accepts any key length");
        mac.update(code.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    /// Verify a code matches its hash using constant-time comparison
    fn verify_code_hash(&self, code: &str, hash: &str) -> bool {
        use subtle::ConstantTimeEq;
        let expected = self.hash_code(code);
        expected.as_bytes().ct_eq(hash.as_bytes()).into()
    }

    /// Create a new verification code or return existing valid one
    ///
    /// Returns the plaintext code (which should be emailed to the user)
    pub async fn create_code(
        &self,
        db: &PgPool,
        email_address_id: i64,
    ) -> Result<String, VerificationError> {
        let code = Self::generate_code();
        let code_hash = self.hash_code(&code);
        let now = Timestamp::now();
        let expires_at = now
            .checked_add(Span::new().hours(self.config.code_expiry_hours))
            .map_err(|e| {
                VerificationError::Internal(format!("Failed to calculate expiration time: {}", e))
            })?;

        // Convert to chrono for database
        let expires_at_chrono = DateTime::from_timestamp(expires_at.as_second(), 0)
            .ok_or_else(|| VerificationError::Internal("Invalid timestamp".to_string()))?;

        sqlx::query!(
            "insert into email_verification_codes
             (email_address_id, code_hash, expires_at)
             values ($1, $2, $3)
             on conflict (email_address_id)
             where verified_at is null
             do update set
                code_hash = excluded.code_hash,
                expires_at = excluded.expires_at,
                created_at = now(),
                last_sent_at = now(),
                send_count = 1,
                attempt_count = 0",
            email_address_id,
            code_hash,
            expires_at_chrono
        )
        .execute(db)
        .await?;

        Ok(code)
    }

    /// Verify a code and mark the email address as verified
    pub async fn verify_code(
        &self,
        db: &PgPool,
        email_address_id: i64,
        code: &str,
    ) -> Result<(), VerificationError> {
        // Get active code record
        let record = sqlx::query!(
            "select code_hash, expires_at, attempt_count
             from email_verification_codes
             where email_address_id = $1 and verified_at is null",
            email_address_id
        )
        .fetch_optional(db)
        .await?
        .ok_or(VerificationError::InvalidCode)?;

        // Check expiry
        let expires_at = Timestamp::from_second(record.expires_at.timestamp()).unwrap();
        if expires_at < Timestamp::now() {
            return Err(VerificationError::Expired);
        }

        // Check attempts
        if record.attempt_count >= self.config.max_verification_attempts {
            return Err(VerificationError::TooManyAttempts);
        }

        // Verify hash
        if !self.verify_code_hash(code, &record.code_hash) {
            // Increment attempt count
            sqlx::query!(
                "update email_verification_codes
                 set attempt_count = attempt_count + 1
                 where email_address_id = $1",
                email_address_id
            )
            .execute(db)
            .await?;

            return Err(VerificationError::InvalidCode);
        }

        // Mark as verified in transaction
        let mut tx = db.begin().await?;

        sqlx::query!(
            "update email_verification_codes
             set verified_at = now()
             where email_address_id = $1",
            email_address_id
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "insert into email_verifications (email_address_id)
             values ($1)
             on conflict do nothing",
            email_address_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Check if resend is allowed based on rate limits
    pub async fn check_resend_limits(
        &self,
        db: &PgPool,
        email_address_id: i64,
        ip: Option<IpAddr>,
    ) -> Result<(), RateLimitError> {
        // Check cooldown based on last_sent_at
        let last_sent = sqlx::query_scalar!(
            "select last_sent_at
             from email_verification_codes
             where email_address_id = $1
               and verified_at is null",
            email_address_id
        )
        .fetch_optional(db)
        .await?;

        if let Some(last_sent) = last_sent {
            let last_sent_ts = Timestamp::from_second(last_sent.timestamp()).unwrap();
            let now = Timestamp::now();
            let elapsed = now.duration_since(last_sent_ts).as_secs();
            if elapsed < self.config.resend_cooldown_secs {
                let remaining = (self.config.resend_cooldown_secs - elapsed) as u32;
                return Err(RateLimitError::Cooldown(remaining));
            }
        }

        // Check resends per hour for this email
        let now = Timestamp::now();
        let one_hour_ago = now
            .checked_sub(Span::new().hours(1))
            .map_err(|_| RateLimitError::TooManyRequests)?;
        let one_hour_ago_chrono = DateTime::from_timestamp(one_hour_ago.as_second(), 0)
            .ok_or(RateLimitError::TooManyRequests)?;
        let email_resends = sqlx::query_scalar!(
            "select count(*)
             from email_verification_resend_log
             where email_address_id = $1
               and created_at > $2",
            email_address_id,
            one_hour_ago_chrono
        )
        .fetch_one(db)
        .await?;

        if email_resends.unwrap_or(0) >= self.config.max_resends_per_hour {
            return Err(RateLimitError::TooManyRequests);
        }

        // Check resends per hour for this IP if provided
        if let Some(ip_addr) = ip {
            let ip_network: IpNetwork = ip_addr.into();
            let ip_resends = sqlx::query_scalar!(
                "select count(*)
                 from email_verification_resend_log
                 where ip_address = $1
                   and created_at > $2",
                ip_network as _,
                one_hour_ago_chrono
            )
            .fetch_one(db)
            .await?;

            // Allow more resends per IP to handle multiple users on same network
            if ip_resends.unwrap_or(0) >= self.config.max_resends_per_hour * 2 {
                return Err(RateLimitError::TooManyRequests);
            }
        }

        Ok(())
    }

    /// Log a resend attempt and update the code record
    pub async fn log_resend(
        &self,
        db: &PgPool,
        email_address_id: i64,
        ip: Option<IpAddr>,
    ) -> Result<(), VerificationError> {
        let mut tx = db.begin().await?;

        // Log resend
        let ip_network = ip.map(IpNetwork::from);
        sqlx::query!(
            "insert into email_verification_resend_log
             (email_address_id, ip_address)
             values ($1, $2)",
            email_address_id,
            ip_network as _
        )
        .execute(&mut *tx)
        .await?;

        // Update last_sent_at and send_count
        sqlx::query!(
            "update email_verification_codes
             set last_sent_at = now(),
                 send_count = send_count + 1
             where email_address_id = $1
               and verified_at is null",
            email_address_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Get the timestamp when the next resend will be allowed
    pub async fn next_resend_at(
        &self,
        db: &PgPool,
        email_address_id: i64,
    ) -> Result<Option<Timestamp>, VerificationError> {
        let last_sent = sqlx::query_scalar!(
            "select last_sent_at
             from email_verification_codes
             where email_address_id = $1
               and verified_at is null",
            email_address_id
        )
        .fetch_optional(db)
        .await?;

        Ok(last_sent.and_then(|ts| {
            let timestamp = Timestamp::from_second(ts.timestamp()).ok()?;
            timestamp
                .checked_add(Span::new().seconds(self.config.resend_cooldown_secs))
                .ok()
        }))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum VerificationError {
    #[error("Invalid or incorrect code")]
    InvalidCode,

    #[error("Too many verification attempts")]
    TooManyAttempts,

    #[error("Code has expired")]
    Expired,

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum RateLimitError {
    #[error("Please wait {0} seconds before requesting another code")]
    Cooldown(u32),

    #[error("Too many resend requests")]
    TooManyRequests,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
