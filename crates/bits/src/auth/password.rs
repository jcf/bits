//! Password management.

use super::core::AuthError;
#[cfg(feature = "server")]
use super::core::{AuthSession, Verified};
#[cfg(feature = "server")]
use bits_domain::{Email, Password, PasswordHash, UserId};
use dioxus::prelude::*;
#[cfg(feature = "server")]
use dioxus::server::axum::extract::Extension;
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use sqlx::PgPool;

#[derive(Clone, Serialize, Deserialize)]
pub struct ChangePasswordForm {
    pub current_password: String, // Raw string from form
    pub new_password: String,     // Raw string from form
    pub confirm_password: String, // Raw string from form
}

// Custom Debug to avoid logging passwords
impl std::fmt::Debug for ChangePasswordForm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChangePasswordForm")
            .field("current_password", &"<redacted>")
            .field("new_password", &"<redacted>")
            .field("confirm_password", &"<redacted>")
            .finish()
    }
}

#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct LoginData {
    user_id: UserId,
    password_hash: PasswordHash,
    verified: bool,
}

#[cfg(feature = "server")]
async fn load_login_data(db: &PgPool, email: &Email) -> anyhow::Result<Option<LoginData>> {
    use anyhow::Context;

    sqlx::query_as::<_, LoginData>(
        "select user_id, password_hash, verified from logins where email = $1 limit 1",
    )
    .bind(email.as_str())
    .fetch_optional(db)
    .await
    .context("Failed to query login data")
}

#[cfg(feature = "server")]
async fn update_password_hash(
    db: &PgPool,
    user_id: UserId,
    password_hash: &PasswordHash,
) -> anyhow::Result<()> {
    use anyhow::Context;
    use secrecy::ExposeSecret;

    sqlx::query!(
        "update users set password_hash = $1 where id = $2",
        password_hash.expose_secret(),
        user_id.get()
    )
    .execute(db)
    .await
    .context("Failed to update password hash")?;

    Ok(())
}

#[cfg(feature = "server")]
async fn invalidate_other_sessions(
    db: &PgPool,
    session_store: &std::sync::Arc<
        tokio::sync::Mutex<bits_axum_session_sqlx::SessionPgSessionStore>,
    >,
    user_id: UserId,
    current_session_id: &str,
) -> anyhow::Result<()> {
    use anyhow::Context;

    let count_before = sqlx::query_scalar!(
        "select count(*) from sessions where user_id = $1",
        user_id.get()
    )
    .fetch_one(db)
    .await
    .context("Failed to count sessions before deletion")?;

    tracing::debug!(
        user_id = ?user_id,
        current_session_id = %current_session_id,
        total_sessions = count_before,
        "Invalidating other sessions (pre-deletion count)"
    );

    tracing::info!(
        user_id = ?user_id,
        current_session_id = %current_session_id,
        total_sessions = count_before,
        "Invalidating other sessions"
    );

    let result = sqlx::query!(
        "delete from sessions where user_id = $1 and id != $2",
        user_id.get(),
        current_session_id
    )
    .execute(db)
    .await
    .context("Failed to invalidate other sessions")?;

    // Clear session cache so deleted sessions aren't served from memory
    session_store.lock().await.clear().await;

    let count_after = sqlx::query_scalar!(
        "select count(*) from sessions where user_id = $1",
        user_id.get()
    )
    .fetch_one(db)
    .await
    .context("Failed to count sessions after deletion")?;

    tracing::debug!(
        deleted = result.rows_affected(),
        remaining = count_after,
        "Sessions deleted (post-deletion count)"
    );

    tracing::info!(
        deleted = result.rows_affected(),
        remaining = count_after,
        "Sessions deleted"
    );

    Ok(())
}

// TODO Use #[patch] when Dioxus next ships
//
// https://github.com/DioxusLabs/dioxus/commit/57e3543c6475b5f6af066774d2152a6dd6351196
#[post("/api/passwords", verified: Verified, auth: AuthSession, state: Extension<crate::AppState>)]
pub async fn change_password(
    form: dioxus::fullstack::Form<ChangePasswordForm>,
) -> Result<(), AuthError> {
    #[cfg(feature = "server")]
    {
        let user = &verified.0;

        if form.0.new_password != form.0.confirm_password {
            return Err(AuthError::Internal("Passwords do not match".to_string()));
        }

        // Wrap passwords in domain types
        let current_password = Password::new(form.0.current_password.clone());
        let new_password = Password::new(form.0.new_password.clone());

        let login_data = load_login_data(&state.db, &user.email)
            .await
            .map_err(|e| AuthError::Internal(format!("Failed to load login data: {}", e)))?;
        let login_data = login_data.ok_or(AuthError::InvalidCredentials)?;

        state
            .password_service
            .verify_password(&current_password, &login_data.password_hash)
            .map_err(|_| AuthError::InvalidCredentials)?;

        let new_hash = state
            .password_service
            .hash_password(&new_password)
            .map_err(|e| AuthError::Internal(format!("Failed to hash password: {}", e)))?;
        update_password_hash(&state.db, user.id, &new_hash)
            .await
            .map_err(|e| AuthError::Internal(format!("Failed to update password: {}", e)))?;

        let session_id = auth.session.get_session_id();
        tracing::debug!(
            user_id = %user.id,
            session_id = %session_id,
            "Password changed, invalidating other sessions"
        );

        invalidate_other_sessions(&state.db, &state.session_store, user.id, &session_id)
            .await
            .map_err(|e| {
                AuthError::Internal(format!("Failed to invalidate other sessions: {}", e))
            })?;

        // Clear user from auth cache so other sessions can't use cached auth
        auth.cache_clear_user(user.id);
        tracing::info!(user_id = %user.id, "Cleared user from auth cache");
    }
    Ok(())
}
