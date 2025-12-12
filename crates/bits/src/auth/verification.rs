//! Email verification.

use super::core::AuthError;
#[cfg(feature = "server")]
use super::core::AuthSession;
use dioxus::prelude::*;
#[cfg(feature = "server")]
use dioxus::server::axum::extract::Extension;
#[cfg(feature = "server")]
use jiff::Timestamp;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerifyEmailForm {
    pub email: String,
    pub code: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResendForm {
    pub email: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResendResponse {
    pub next_resend_at: i64,
}

#[post("/api/email-verifications", auth: AuthSession, state: Extension<crate::AppState>)]
pub async fn verify_email_code(
    form: dioxus::fullstack::Form<VerifyEmailForm>,
) -> Result<(), AuthError> {
    #[cfg(feature = "server")]
    {
        // Look up user by email
        let user_id = sqlx::query_scalar!(
            "select user_id from email_addresses where address = $1 and valid_to = 'infinity' limit 1",
            form.0.email
        )
        .fetch_optional(&state.db)
        .await?
        .ok_or(AuthError::InvalidCredentials)?;

        let email_address_id = super::registration::get_email_address_id(&state.db, user_id.into())
            .await
            .map_err(|e| AuthError::Internal(format!("Failed to get email address ID: {}", e)))?;

        // Verify the code
        state
            .email_verification
            .verify_code(&state.db, email_address_id, &form.0.code)
            .await
            .map_err(|e| match e {
                crate::verification::VerificationError::InvalidCode => {
                    AuthError::Internal("Invalid verification code".to_string())
                }
                crate::verification::VerificationError::TooManyAttempts => {
                    AuthError::Internal("Too many attempts. Please request a new code.".to_string())
                }
                crate::verification::VerificationError::Expired => AuthError::Internal(
                    "Verification code has expired. Please request a new code.".to_string(),
                ),
                crate::verification::VerificationError::Internal(_)
                | crate::verification::VerificationError::Database(_) => {
                    AuthError::Internal("Verification failed".to_string())
                }
            })?;

        tracing::info!(
            user_id = user_id,
            email = %form.0.email,
            "Email verified successfully"
        );

        // Clear user from cache so next request loads updated verified status
        auth.cache_clear_user(user_id.into());

        Ok(())
    }
    #[cfg(not(feature = "server"))]
    Ok(())
}

#[post("/api/email-verifications/resend", _auth: AuthSession, state: Extension<crate::AppState>)]
pub async fn resend_verification_code(
    form: dioxus::fullstack::Form<ResendForm>,
) -> Result<ResendResponse, AuthError> {
    #[cfg(feature = "server")]
    {
        // Look up user by email
        let user_id = sqlx::query_scalar!(
            "select user_id from email_addresses where address = $1 and valid_to = 'infinity' limit 1",
            form.0.email
        )
        .fetch_optional(&state.db)
        .await?
        .ok_or(AuthError::InvalidCredentials)?;

        let email_address_id = super::registration::get_email_address_id(&state.db, user_id.into())
            .await
            .map_err(|e| AuthError::Internal(format!("Failed to get email address ID: {}", e)))?;

        // Check rate limits (IP is None for now - Phase 6 will add IP extraction)
        state
            .email_verification
            .check_resend_limits(&state.db, email_address_id, None)
            .await
            .map_err(|e| match e {
                crate::verification::RateLimitError::Cooldown(secs) => {
                    AuthError::Internal(format!(
                        "Please wait {} seconds before requesting another code",
                        secs
                    ))
                }
                crate::verification::RateLimitError::TooManyRequests => AuthError::Internal(
                    "Too many resend requests. Please try again later.".to_string(),
                ),
                crate::verification::RateLimitError::Database(e) => {
                    AuthError::Internal(format!("Database error: {}", e))
                }
            })?;

        // Get or create code (same code if still valid)
        let code = state
            .email_verification
            .create_code(&state.db, email_address_id)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        // Log the resend
        state
            .email_verification
            .log_resend(&state.db, email_address_id, None)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        tracing::info!(
            user_id = user_id,
            email = %form.0.email,
            code = %code,
            "Verification code resent"
        );

        // Get next resend time
        let next_resend = state
            .email_verification
            .next_resend_at(&state.db, email_address_id)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?
            .unwrap_or_else(Timestamp::now);

        Ok(ResendResponse {
            next_resend_at: next_resend.as_second(),
        })
    }
    #[cfg(not(feature = "server"))]
    Ok(ResendResponse { next_resend_at: 0 })
}
