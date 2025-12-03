//! CSRF (Cross-Site Request Forgery) protection
//!
//! This module implements session-based CSRF protection with session-lifetime tokens:
//! - Tokens stored in database (sessions.csrf_token column)
//! - Tokens persist for session lifetime (not deleted after each use)
//! - Supports multi-tab UX and distributed systems
//! - Timing-safe comparison to prevent timing attacks
//! - Works alongside SameSite=Strict cookies for defense in depth

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;
use sqlx::PgPool;

const TOKEN_LENGTH: usize = 32;

/// Generate a new CSRF token (32 random bytes, base64 encoded)
pub fn generate_token() -> String {
    let mut token_bytes = [0u8; TOKEN_LENGTH];
    rand::rng().fill_bytes(&mut token_bytes);
    BASE64.encode(token_bytes)
}

/// Store a CSRF token in the database for the given session
pub async fn store_token(db: &PgPool, session_id: &str, token: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "update sessions set csrf_token = $1 where id = $2",
        token,
        session_id
    )
    .execute(db)
    .await?;
    Ok(())
}

/// Retrieve the CSRF token for a given session
pub async fn get_token(db: &PgPool, session_id: &str) -> Result<Option<String>, sqlx::Error> {
    let result = sqlx::query!(
        "select csrf_token from sessions where id = $1",
        session_id
    )
    .fetch_optional(db)
    .await?;

    Ok(result.and_then(|r| r.csrf_token))
}

/// Delete the CSRF token for a given session (used during logout/session cleanup)
pub async fn delete_token(db: &PgPool, session_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "update sessions set csrf_token = null where id = $1",
        session_id
    )
    .execute(db)
    .await?;
    Ok(())
}

/// Validate token format (length and base64 encoding)
/// Returns true if token appears valid, false otherwise
/// This is a fast check before expensive database lookup
pub fn is_valid_format(token: &str) -> bool {
    // 32 bytes base64 encoded = 44 characters (with padding)
    if token.len() != 44 {
        return false;
    }

    // Check if valid base64
    BASE64.decode(token).is_ok()
}

/// Verify a CSRF token using timing-safe comparison
pub fn verify_token(expected: &str, provided: &str) -> bool {
    use subtle::ConstantTimeEq;

    if expected.len() != provided.len() {
        return false;
    }

    expected.as_bytes().ct_eq(provided.as_bytes()).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_token() {
        let token1 = generate_token();
        let token2 = generate_token();

        // Tokens should be non-empty
        assert!(!token1.is_empty());
        assert!(!token2.is_empty());

        // Tokens should be unique
        assert_ne!(token1, token2);

        // Tokens should be base64 encoded (roughly 44 chars for 32 bytes)
        assert!(token1.len() > 40);
    }

    #[test]
    fn test_verify_token() {
        let token = "test_token_123";

        assert!(verify_token(token, token));
        assert!(!verify_token(token, "different_token"));
        assert!(!verify_token(token, ""));
        assert!(!verify_token("", token));
    }
}
