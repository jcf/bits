//! Configuration for authentication rate limiting

#[derive(Clone, Debug)]
pub struct AuthRateLimitConfig {
    /// Maximum authentication attempts per IP address within the time window
    pub ip_attempts_per_window: u32,
    /// Time window for IP-based rate limiting (in seconds)
    pub ip_window_secs: i64,
    /// Maximum authentication attempts per email address per hour
    pub email_attempts_per_hour: u32,
}

impl AuthRateLimitConfig {
    /// Configuration for login endpoints (stricter limits)
    pub fn for_login() -> Self {
        Self {
            ip_attempts_per_window: 10,
            ip_window_secs: 15 * 60, // 15 minutes
            email_attempts_per_hour: 5,
        }
    }

    /// Configuration for registration endpoints (more lenient)
    pub fn for_registration() -> Self {
        Self {
            ip_attempts_per_window: 20,
            ip_window_secs: 15 * 60, // 15 minutes
            email_attempts_per_hour: 2,
        }
    }
}
