//! Core authentication types and extractors.
//!
//! # Authorization Levels
//!
//! Three extractors provide different authorization levels:
//!
//! - **`AuthSession`**: Full session control (login, logout, cache management)
//!   - Use when you need session lifecycle methods
//!   - Example: `auth.login_user()`, `auth.session.renew()`, `auth.cache_clear_user()`
//!
//! - **`Authenticated(User)`**: Requires authentication, allows unverified users
//!   - Use when you need to know who the user is but verification isn't required
//!   - Automatically returns 401 if not authenticated
//!
//! - **`Verified(User)`**: Requires email verification
//!   - Use when the endpoint requires a verified user
//!   - Automatically returns 403 if not verified

#[cfg(feature = "server")]
use axum_session_auth::{Authentication, HasPermission};
use bits_domain::{Email, UserId};
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use sqlx::PgPool;

#[cfg(feature = "server")]
pub type AuthSession =
    axum_session_auth::AuthSession<User, UserId, bits_axum_session_sqlx::SessionPgPool, PgPool>;

#[derive(thiserror::Error, Debug, Serialize, Deserialize)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Email not verified")]
    EmailNotVerified,

    #[error("Email already registered")]
    EmailAlreadyRegistered,

    #[error("Forbidden")]
    Forbidden,

    #[error("Internal error")]
    Internal(String),
}

impl From<dioxus::prelude::ServerFnError> for AuthError {
    fn from(err: dioxus::prelude::ServerFnError) -> Self {
        AuthError::Internal(err.to_string())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<sqlx::Error> for AuthError {
    fn from(err: sqlx::Error) -> Self {
        AuthError::Internal(format!("Database error: {}", err))
    }
}

impl dioxus::fullstack::AsStatusCode for AuthError {
    fn as_status_code(&self) -> dioxus::fullstack::StatusCode {
        match self {
            AuthError::InvalidCredentials => dioxus::fullstack::StatusCode::UNAUTHORIZED,
            AuthError::EmailNotVerified => dioxus::fullstack::StatusCode::FORBIDDEN,
            AuthError::EmailAlreadyRegistered => dioxus::fullstack::StatusCode::BAD_REQUEST,
            AuthError::Forbidden => dioxus::fullstack::StatusCode::FORBIDDEN,
            AuthError::Internal(_) => dioxus::fullstack::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[cfg(feature = "server")]
impl axum_core::response::IntoResponse for AuthError {
    fn into_response(self) -> axum_core::response::Response {
        use dioxus::server::axum::http::StatusCode;

        let status = match self {
            AuthError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            AuthError::EmailNotVerified => StatusCode::FORBIDDEN,
            AuthError::EmailAlreadyRegistered => StatusCode::BAD_REQUEST,
            AuthError::Forbidden => StatusCode::FORBIDDEN,
            AuthError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, self.to_string()).into_response()
    }
}

#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub email: Email,
    #[serde(default)]
    pub verified: bool,
}

// PII-safe Debug implementation
impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("email", &self.email)
            .field("verified", &self.verified)
            .finish()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SessionState {
    Anonymous,
    Authenticated(User),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UserState {
    Anonymous,
    Unverified(User),
    Verified(User),
}

#[cfg(feature = "server")]
#[async_trait::async_trait]
impl Authentication<User, UserId, PgPool> for User {
    async fn load_user(userid: UserId, pool: Option<&PgPool>) -> Result<User, anyhow::Error> {
        let Some(pool) = pool else {
            return Err(anyhow::anyhow!("No database pool"));
        };

        let user = sqlx::query_as!(
            User,
            "select user_id as \"id!\", email as \"email!\", verified as \"verified!\" from logins where user_id = $1 limit 1",
            userid.get()
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    fn is_authenticated(&self) -> bool {
        self.id.get() > 0
    }

    fn is_active(&self) -> bool {
        self.id.get() > 0
    }

    fn is_anonymous(&self) -> bool {
        self.id.get() < 0
    }
}

#[cfg(feature = "server")]
#[async_trait::async_trait]
impl HasPermission<PgPool> for User {
    async fn has(&self, _perm: &str, _pool: &Option<&PgPool>) -> bool {
        self.is_authenticated()
    }
}

/// Extractor that requires authentication but allows unverified users
///
/// Use this for endpoints that need to know who the user is but don't require
/// email verification (e.g., email verification endpoints themselves).
/// Returns AuthError::InvalidCredentials if not authenticated.
#[cfg(feature = "server")]
pub struct Authenticated(pub User);

#[cfg(feature = "server")]
impl<S> axum_core::extract::FromRequestParts<S> for Authenticated
where
    S: Send + Sync,
    crate::AppState: axum_core::extract::FromRef<S>,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut dioxus::server::axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth = AuthSession::from_request_parts(parts, state)
            .await
            .map_err(|_| AuthError::InvalidCredentials)?;

        let user = auth
            .current_user
            .filter(|u| u.is_authenticated())
            .ok_or(AuthError::InvalidCredentials)?;

        Ok(Authenticated(user))
    }
}

/// Extractor that requires email verification
///
/// Use this instead of AuthSession for endpoints that require verified users.
/// Returns AuthError::EmailNotVerified if user is authenticated but not verified.
#[cfg(feature = "server")]
pub struct Verified(pub User);

#[cfg(feature = "server")]
impl<S> axum_core::extract::FromRequestParts<S> for Verified
where
    S: Send + Sync,
    crate::AppState: axum_core::extract::FromRef<S>,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut dioxus::server::axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth = AuthSession::from_request_parts(parts, state)
            .await
            .map_err(|_| AuthError::InvalidCredentials)?;

        let user = auth
            .current_user
            .filter(|u| u.is_authenticated())
            .ok_or(AuthError::InvalidCredentials)?;

        if !user.verified {
            return Err(AuthError::EmailNotVerified);
        }

        Ok(Verified(user))
    }
}
