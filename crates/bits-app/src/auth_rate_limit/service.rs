//! Service layer for authentication rate limiting

use super::{config::AuthRateLimitConfig, error::RateLimitError, storage};
use jiff::{Span, Timestamp};
use sqlx::PgPool;
use std::net::IpAddr;

#[derive(Clone)]
pub struct AuthRateLimitService {
    login_config: AuthRateLimitConfig,
    registration_config: AuthRateLimitConfig,
    ip_tracker: storage::IpAttemptTracker,
}

impl AuthRateLimitService {
    pub fn new() -> Self {
        Self {
            login_config: AuthRateLimitConfig::for_login(),
            registration_config: AuthRateLimitConfig::for_registration(),
            ip_tracker: storage::IpAttemptTracker::new(),
        }
    }

    pub fn with_configs(
        login_config: AuthRateLimitConfig,
        registration_config: AuthRateLimitConfig,
    ) -> Self {
        Self {
            login_config,
            registration_config,
            ip_tracker: storage::IpAttemptTracker::new(),
        }
    }

    /// Check rate limits for login endpoint
    pub async fn check_login_limits(
        &self,
        db: &PgPool,
        ip: IpAddr,
        email: &str,
    ) -> Result<(), RateLimitError> {
        self.check_limits(db, ip, email, &self.login_config).await
    }

    /// Check rate limits for registration endpoint
    pub async fn check_registration_limits(
        &self,
        db: &PgPool,
        ip: IpAddr,
        email: &str,
    ) -> Result<(), RateLimitError> {
        self.check_limits(db, ip, email, &self.registration_config)
            .await
    }

    /// Core rate limiting logic
    ///
    /// Checks both IP-based (per-instance) and email-based (database) limits.
    /// Returns email limit errors first (to prevent enumeration via timing).
    async fn check_limits(
        &self,
        db: &PgPool,
        ip: IpAddr,
        email: &str,
        config: &AuthRateLimitConfig,
    ) -> Result<(), RateLimitError> {
        // Check email-based limits first (database query, slower)
        // This prevents timing attacks that could enumerate valid emails
        let email_attempts = storage::count_email_attempts(db, email, 1).await?;

        if email_attempts >= config.email_attempts_per_hour as i64 {
            return Err(RateLimitError::EmailLimitExceeded);
        }

        // Check IP-based limits (in-memory, fast)
        let ip_attempts = self.ip_tracker.count_attempts(ip, config.ip_window_secs);

        if ip_attempts >= config.ip_attempts_per_window {
            let now = Timestamp::now().as_second();
            let cutoff = now - config.ip_window_secs;
            let retry_after = config.ip_window_secs as u64;

            return Err(RateLimitError::IpLimitExceeded(retry_after));
        }

        Ok(())
    }

    /// Record an authentication attempt
    ///
    /// Updates both in-memory IP tracker and database email record.
    pub async fn record_attempt(
        &self,
        db: &PgPool,
        ip: IpAddr,
        email: &str,
        endpoint: &str,
    ) -> Result<(), RateLimitError> {
        // Record in-memory
        self.ip_tracker.record_attempt(ip);

        // Record in database
        storage::record_attempt(db, email, endpoint, Some(ip)).await?;

        Ok(())
    }

    /// Clean up old IP tracking data (for maintenance/testing)
    pub fn cleanup_ip_tracker(&self, window_secs: i64) {
        self.ip_tracker.cleanup(window_secs);
    }
}

impl Default for AuthRateLimitService {
    fn default() -> Self {
        Self::new()
    }
}
