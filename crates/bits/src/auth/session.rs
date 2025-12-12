//! Session management: login, logout, session state.

#[cfg(feature = "server")]
use super::core::AuthSession;
use super::core::{AuthError, SessionState, User};
#[cfg(feature = "server")]
use axum_session_auth::Authentication;
#[cfg(feature = "server")]
use bits_domain::{Email, Password, PasswordHash, UserId};
use dioxus::prelude::*;
#[cfg(feature = "server")]
use dioxus::server::axum::extract::Extension;
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use sqlx::PgPool;

#[cfg(feature = "server")]
use crate::metrics::{LoginAttempt, LogoutAttempt, RecordMetrics};

#[derive(Clone, Serialize, Deserialize)]
pub struct AuthForm {
    pub email: String,    // Raw string from form, validated in handler
    pub password: String, // Raw string from form
}

// Custom Debug to avoid logging passwords
impl std::fmt::Debug for AuthForm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthForm")
            .field("email", &self.email)
            .field("password", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AuthResponse {
    Success(User),
    NeedsVerification { email: String },
}

#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
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

#[post("/api/sessions", auth: AuthSession, state: Extension<crate::AppState>)]
pub async fn auth(form: dioxus::fullstack::Form<AuthForm>) -> Result<User, AuthError> {
    (async {
        // Parse and validate email at boundary
        let email = Email::parse(&form.0.email).map_err(|_| AuthError::InvalidCredentials)?;

        // Wrap password in domain type
        let password = Password::new(form.0.password.clone());

        let user = load_login_data(&state.db, &email)
            .await
            .map_err(|e| AuthError::Internal(format!("Failed to load login data: {}", e)))?;

        let user = match user {
            Some(u) => u,
            None => {
                return Err(AuthError::InvalidCredentials);
            }
        };

        state
            .password_service
            .verify_password(&password, &user.password_hash)
            .map_err(|_| AuthError::InvalidCredentials)?;

        auth.session.renew();
        auth.login_user(user.user_id);

        // Clear user from auth cache so next request loads fresh from database
        // This ensures verification status and other user attributes are up-to-date
        auth.cache_clear_user(user.user_id);

        Ok(User {
            id: user.user_id,
            email,
            verified: user.verified,
        })
    }
    .await)
        .record(LoginAttempt)
}

#[delete("/api/session", auth: AuthSession)]
pub async fn sign_out() -> Result<()> {
    auth.logout_user();
    Ok(()).record(LogoutAttempt)
}

#[get("/api/session", auth: AuthSession)]
pub async fn get_session() -> Result<SessionState> {
    if let Some(user) = auth.current_user {
        if user.is_authenticated() {
            return Ok(SessionState::Authenticated(user));
        }
    }
    Ok(SessionState::Anonymous)
}

#[get("/api/realm", realm: crate::Realm)]
pub async fn get_realm() -> Result<crate::Realm> {
    tracing::debug!("get_realm called with realm: {:?}", realm);
    if matches!(realm, crate::Realm::NotFound) {
        tracing::warn!("Realm is NotFound, setting 404 status");
        dioxus::fullstack::FullstackContext::commit_http_status(
            dioxus::fullstack::StatusCode::NOT_FOUND,
            None,
        );
    }
    Ok(realm)
}
