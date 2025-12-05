//! Error types for authentication rate limiting

use std::time::Duration;

#[derive(thiserror::Error, Debug)]
pub enum RateLimitError {
    #[error(
        "Too many authentication attempts from this IP address. Please try again in {0} seconds."
    )]
    IpLimitExceeded(u64),

    #[error("Too many authentication attempts for this email address. Please try again later.")]
    EmailLimitExceeded,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl RateLimitError {
    /// Calculate retry-after duration for rate limit errors
    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            RateLimitError::IpLimitExceeded(secs) => Some(Duration::from_secs(*secs)),
            RateLimitError::EmailLimitExceeded => Some(Duration::from_secs(3600)), // 1 hour
            _ => None,
        }
    }
}

#[cfg(feature = "server")]
impl axum_core::response::IntoResponse for RateLimitError {
    fn into_response(self) -> axum_core::response::Response {
        use dioxus::server::axum::http::{header, StatusCode};

        match &self {
            // IP-based limits return 429 (obvious to attacker anyway)
            RateLimitError::IpLimitExceeded(secs) => {
                crate::metrics::record_rate_limit_hit("ip", "auth");
                let mut response =
                    (StatusCode::TOO_MANY_REQUESTS, self.to_string()).into_response();
                if let Ok(retry_after) = secs.to_string().try_into() {
                    response
                        .headers_mut()
                        .insert(header::RETRY_AFTER, retry_after);
                }
                response
            }
            // Email-based limits return 401 (prevent email enumeration)
            // This makes it look like invalid credentials
            RateLimitError::EmailLimitExceeded => {
                crate::metrics::record_rate_limit_hit("email", "auth");
                (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()
            }
            RateLimitError::Database(_) | RateLimitError::Internal(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
            }
        }
    }
}
