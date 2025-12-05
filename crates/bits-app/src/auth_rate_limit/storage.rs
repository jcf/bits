//! Storage layer for authentication rate limiting
//!
//! Handles both in-memory IP tracking and database-backed email tracking.

use dashmap::DashMap;
use ipnetwork::IpNetwork;
use jiff::{Span, Timestamp};
use sqlx::PgPool;
use std::net::IpAddr;
use std::sync::Arc;

/// In-memory tracker for IP-based rate limiting
///
/// Uses DashMap for concurrent access across multiple requests.
/// Stores timestamps of recent attempts, automatically filtered by time window.
#[derive(Clone)]
pub struct IpAttemptTracker {
    /// Map of IP address to list of attempt timestamps
    attempts: Arc<DashMap<IpAddr, Vec<i64>>>,
}

impl IpAttemptTracker {
    pub fn new() -> Self {
        Self {
            attempts: Arc::new(DashMap::new()),
        }
    }

    /// Record an attempt from the given IP address
    pub fn record_attempt(&self, ip: IpAddr) {
        let now = Timestamp::now().as_second();
        self.attempts
            .entry(ip)
            .and_modify(|attempts| attempts.push(now))
            .or_insert_with(|| vec![now]);
    }

    /// Count attempts from the given IP within the time window
    pub fn count_attempts(&self, ip: IpAddr, window_secs: i64) -> u32 {
        let now = Timestamp::now().as_second();
        let cutoff = now - window_secs;

        self.attempts
            .get_mut(&ip)
            .map(|mut entry| {
                // Remove old attempts
                entry.value_mut().retain(|&timestamp| timestamp > cutoff);
                entry.value().len() as u32
            })
            .unwrap_or(0)
    }

    /// Clear old attempts (for cleanup/testing)
    pub fn cleanup(&self, window_secs: i64) {
        let now = Timestamp::now().as_second();
        let cutoff = now - window_secs;

        self.attempts.retain(|_ip, attempts| {
            attempts.retain(|&timestamp| timestamp > cutoff);
            !attempts.is_empty()
        });
    }
}

impl Default for IpAttemptTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Count authentication attempts for a given email within the past hour
pub async fn count_email_attempts(
    db: &PgPool,
    email: &str,
    hours: i64,
) -> Result<i64, sqlx::Error> {
    let now = Timestamp::now();
    let cutoff = now
        .checked_sub(Span::new().hours(hours))
        .map_err(|e| sqlx::Error::Protocol(format!("Failed to calculate cutoff time: {}", e)))?;

    let cutoff_timestamp = cutoff.as_second() as f64;

    let count = sqlx::query_scalar!(
        "select count(*) from auth_attempts where email = $1 and created_at > to_timestamp($2)",
        email,
        cutoff_timestamp
    )
    .fetch_one(db)
    .await?;

    Ok(count.unwrap_or(0))
}

/// Record an authentication attempt in the database
pub async fn record_attempt(
    db: &PgPool,
    email: &str,
    endpoint: &str,
    ip: Option<IpAddr>,
) -> Result<(), sqlx::Error> {
    let ip_network = ip.map(IpNetwork::from);

    sqlx::query!(
        "insert into auth_attempts (email, endpoint, ip_address) values ($1, $2, $3)",
        email,
        endpoint,
        ip_network as _
    )
    .execute(db)
    .await?;

    Ok(())
}
