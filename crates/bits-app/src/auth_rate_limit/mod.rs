//! Authentication rate limiting
//!
//! Provides layered rate limiting for authentication endpoints:
//! - IP-based limits (per-instance, in-memory)
//! - Email-based limits (cross-instance, database-backed)
//!
//! ## Security Considerations
//!
//! - Email-based limits return 401 "Invalid credentials" to prevent email enumeration
//! - IP-based limits return 429 "Too Many Requests" (IP is obvious to attacker)
//! - Email checks happen first to prevent timing attacks
//!
//! ## Usage
//!
//! ```rust,ignore
//! use bits_app::auth_rate_limit::AuthRateLimitService;
//!
//! let service = AuthRateLimitService::new();
//!
//! // Check limits before authentication
//! service.check_login_limits(&db, client_ip, email).await?;
//!
//! // Record attempt after authentication (success or failure)
//! service.record_attempt(&db, client_ip, email, "/api/sessions").await?;
//! ```

pub mod config;
pub mod error;
pub mod service;
pub mod storage;
pub mod types;

pub use config::AuthRateLimitConfig;
pub use error::RateLimitError;
pub use service::AuthRateLimitService;
pub use types::{EmailAddress, Visitor};
