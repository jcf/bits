//! Authentication and authorization.
//!
//! # Authorization Levels
//!
//! - **Public**: No authentication required
//!   - `get_realm` - Get current realm (tenant/platform/demo)
//!   - `get_session` - Check if request is authenticated
//!
//! - **Authenticated**: Requires login, allows unverified users
//!   - `auth` - Sign in
//!   - `join` - Sign up
//!   - `sign_out` - Sign out
//!   - `verify_email_code` - Verify email with code
//!   - `resend_verification_code` - Resend verification code
//!
//! - **Verified**: Requires email verification (uses `RequireVerified` extractor)
//!   - `change_password` - Change user password

#[cfg(feature = "server")]
use axum_session_auth::{Authentication, HasPermission};
#[cfg(feature = "server")]
use dioxus::server::axum::extract::Extension;
#[cfg(feature = "server")]
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use sqlx::PgPool;

use dioxus::prelude::*;

#[cfg(feature = "server")]
pub type AuthSession =
    axum_session_auth::AuthSession<User, i64, bits_axum_session_sqlx::SessionPgPool, PgPool>;

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

impl From<ServerFnError> for AuthError {
    fn from(err: ServerFnError) -> Self {
        AuthError::Internal(err.to_string())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<anyhow::Error> for AuthError {
    fn from(err: anyhow::Error) -> Self {
        AuthError::Internal(format!("{:#}", err))
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
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub email: String,
    #[serde(default)]
    pub verified: bool,
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
impl Default for User {
    fn default() -> Self {
        Self {
            id: -1,
            email: String::new(),
            verified: false,
        }
    }
}

#[cfg(feature = "server")]
#[async_trait::async_trait]
impl Authentication<User, i64, PgPool> for User {
    async fn load_user(userid: i64, pool: Option<&PgPool>) -> Result<User, anyhow::Error> {
        let Some(pool) = pool else {
            return Err(anyhow::anyhow!("No database pool"));
        };

        let user = sqlx::query_as!(
            User,
            "select user_id as \"id!\", email as \"email!\", verified as \"verified!\" from logins where user_id = $1 limit 1",
            userid
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    fn is_authenticated(&self) -> bool {
        self.id > 0
    }

    fn is_active(&self) -> bool {
        self.id > 0
    }

    fn is_anonymous(&self) -> bool {
        self.id < 0
    }
}

#[cfg(feature = "server")]
#[async_trait::async_trait]
impl HasPermission<PgPool> for User {
    async fn has(&self, _perm: &str, _pool: &Option<&PgPool>) -> bool {
        self.is_authenticated()
    }
}

/// Extractor that requires email verification
///
/// Use this instead of AuthSession for endpoints that require verified users.
/// Returns AuthError::EmailNotVerified if user is authenticated but not verified.
#[cfg(feature = "server")]
pub struct RequireVerified(pub User);

#[cfg(feature = "server")]
impl<S> axum_core::extract::FromRequestParts<S> for RequireVerified
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

        Ok(RequireVerified(user))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthForm {
    pub email: String,
    pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JoinForm {
    pub email: String,
    pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChangePasswordForm {
    pub current_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AuthResponse {
    Success(User),
    NeedsVerification { email: String },
}

#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
struct LoginData {
    user_id: i64,
    password_hash: String,
    verified: bool,
}

#[cfg(feature = "server")]
async fn load_login_data(db: &PgPool, email: &str) -> anyhow::Result<Option<LoginData>> {
    use anyhow::Context;

    sqlx::query_as!(
        LoginData,
        "select user_id as \"user_id!\", password_hash as \"password_hash!\", verified as \"verified!\" from logins where email = $1 limit 1",
        email
    )
    .fetch_optional(db)
    .await
    .context("Failed to query login data")
}

#[post("/api/sessions", auth: AuthSession, state: Extension<crate::AppState>)]
pub async fn auth(form: dioxus::fullstack::Form<AuthForm>) -> Result<User, AuthError> {
    let user = load_login_data(&state.db, &form.0.email).await?;

    let user = match user {
        Some(u) => u,
        None => {
            return Err(AuthError::InvalidCredentials);
        }
    };

    state
        .password_service
        .verify_password(&form.0.password, &user.password_hash)
        .map_err(|_| {
            crate::metrics::record_auth_event("login", false);
            AuthError::InvalidCredentials
        })?;

    auth.session.renew();
    auth.login_user(user.user_id);

    // Clear user from auth cache so next request loads fresh from database
    // This ensures verification status and other user attributes are up-to-date
    auth.cache_clear_user(user.user_id);

    crate::metrics::record_auth_event("login", true);

    Ok(User {
        id: user.user_id,
        email: form.0.email.clone(),
        verified: user.verified,
    })
}

#[delete("/api/session", auth: AuthSession)]
pub async fn sign_out() -> Result<()> {
    auth.logout_user();
    #[cfg(feature = "server")]
    crate::metrics::record_auth_event("logout", true);
    Ok(())
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

#[cfg(feature = "server")]
async fn create_user_with_email(
    db: &PgPool,
    email: &str,
    password_hash: &str,
) -> anyhow::Result<i64> {
    use anyhow::Context;

    let mut tx = db.begin().await.context("Failed to begin transaction")?;

    let user_id: i64 =
        sqlx::query_scalar("insert into users (password_hash) values ($1) returning id")
            .bind(password_hash)
            .fetch_one(&mut *tx)
            .await
            .context("Failed to insert user")?;

    sqlx::query("insert into email_addresses (user_id, address) values ($1, $2)")
        .bind(user_id)
        .bind(email)
        .execute(&mut *tx)
        .await
        .context("Failed to insert email address")?;

    tx.commit().await.context("Failed to commit transaction")?;

    Ok(user_id)
}

#[cfg(feature = "server")]
async fn update_password_hash(
    db: &PgPool,
    user_id: i64,
    password_hash: &str,
) -> anyhow::Result<()> {
    use anyhow::Context;

    sqlx::query!(
        "update users set password_hash = $1 where id = $2",
        password_hash,
        user_id
    )
    .execute(db)
    .await
    .context("Failed to update password hash")?;

    Ok(())
}

#[cfg(feature = "server")]
async fn get_email_address_id(db: &PgPool, user_id: i64) -> anyhow::Result<i64> {
    use anyhow::Context;

    let email_address_id = sqlx::query_scalar!(
        "select id from email_addresses where user_id = $1 and valid_to = 'infinity' limit 1",
        user_id
    )
    .fetch_one(db)
    .await
    .context("Failed to get email address for user")?;

    Ok(email_address_id)
}

#[cfg(feature = "server")]
#[allow(dead_code)]
async fn is_email_verified(db: &PgPool, user_id: i64) -> anyhow::Result<bool> {
    use anyhow::Context;

    let verified = sqlx::query_scalar!(
        "select exists(
            select 1
            from email_addresses ea
            join email_verifications ev on ev.email_address_id = ea.id
            where ea.user_id = $1 and ea.valid_to = 'infinity'
        )",
        user_id
    )
    .fetch_one(db)
    .await
    .context("Failed to check email verification status")?;

    Ok(verified.unwrap_or(false))
}

#[cfg(feature = "server")]
async fn invalidate_other_sessions(
    db: &PgPool,
    session_store: &std::sync::Arc<
        tokio::sync::Mutex<bits_axum_session_sqlx::SessionPgSessionStore>,
    >,
    user_id: i64,
    current_session_id: &str,
) -> anyhow::Result<()> {
    use anyhow::Context;

    let count_before =
        sqlx::query_scalar!("select count(*) from sessions where user_id = $1", user_id)
            .fetch_one(db)
            .await
            .context("Failed to count sessions before deletion")?;

    tracing::debug!(
        user_id = user_id,
        current_session_id = %current_session_id,
        total_sessions = count_before,
        "Invalidating other sessions (pre-deletion count)"
    );

    tracing::info!(
        user_id = user_id,
        current_session_id = %current_session_id,
        total_sessions = count_before,
        "Invalidating other sessions"
    );

    let result = sqlx::query!(
        "delete from sessions where user_id = $1 and id != $2",
        user_id,
        current_session_id
    )
    .execute(db)
    .await
    .context("Failed to invalidate other sessions")?;

    // Clear session cache so deleted sessions aren't served from memory
    session_store.lock().await.clear().await;

    let count_after =
        sqlx::query_scalar!("select count(*) from sessions where user_id = $1", user_id)
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

#[post("/api/users", auth: AuthSession, state: Extension<crate::AppState>)]
pub async fn join(form: dioxus::fullstack::Form<JoinForm>) -> Result<User, AuthError> {
    #[cfg(feature = "server")]
    {
        let password_hash = state.password_service.hash_password(&form.0.password)?;

        match create_user_with_email(&state.db, &form.0.email, &password_hash).await {
            Ok(user_id) => {
                crate::metrics::record_auth_event("register", true);

                // Log user in after successful registration
                auth.session.renew();
                auth.login_user(user_id);

                // Generate verification code for new user
                match get_email_address_id(&state.db, user_id).await {
                    Ok(email_address_id) => {
                        match state
                            .email_verification
                            .create_code(&state.db, email_address_id)
                            .await
                        {
                            Ok(code) => {
                                tracing::info!(
                                    user_id = user_id,
                                    email = %form.0.email,
                                    code = %code,
                                    "User registered, verification code generated"
                                );
                            }
                            Err(e) => {
                                tracing::error!(
                                    user_id = user_id,
                                    email = %form.0.email,
                                    error = %e,
                                    "Failed to generate verification code"
                                );
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            user_id = user_id,
                            email = %form.0.email,
                            error = %e,
                            "Failed to get email address id"
                        );
                    }
                }

                Ok(User {
                    id: user_id,
                    email: form.0.email.clone(),
                    verified: false,
                })
            }
            Err(e) => {
                if let Some(source) = e.source() {
                    if let Some(db_err) = source.downcast_ref::<sqlx::Error>() {
                        if let Some(pg_err) = db_err.as_database_error() {
                            if pg_err.code().as_deref() == Some("23P01") {
                                crate::metrics::record_auth_event("register", false);
                                return Err(AuthError::EmailAlreadyRegistered);
                            }
                        }
                    }
                }
                crate::metrics::record_auth_event("register", false);
                Err(AuthError::Internal(format!("{:#}", e)))
            }
        }
    }
    #[cfg(not(feature = "server"))]
    Ok(User::default())
}

// TODO Use #[patch] when Dioxus next ships
//
// https://github.com/DioxusLabs/dioxus/commit/57e3543c6475b5f6af066774d2152a6dd6351196
#[post("/api/passwords", verified: RequireVerified, auth: AuthSession, state: Extension<crate::AppState>)]
pub async fn change_password(
    form: dioxus::fullstack::Form<ChangePasswordForm>,
) -> Result<(), AuthError> {
    #[cfg(feature = "server")]
    {
        let user = &verified.0;

        if form.0.new_password != form.0.confirm_password {
            return Err(AuthError::Internal("Passwords do not match".to_string()));
        }

        let login_data = load_login_data(&state.db, &user.email).await?;
        let login_data = login_data.ok_or(AuthError::InvalidCredentials)?;

        state
            .password_service
            .verify_password(&form.0.current_password, &login_data.password_hash)
            .map_err(|_| AuthError::InvalidCredentials)?;

        let new_hash = state.password_service.hash_password(&form.0.new_password)?;
        update_password_hash(&state.db, user.id, &new_hash).await?;

        let session_id = auth.session.get_session_id();
        tracing::debug!(
            user_id = user.id,
            session_id = %session_id,
            "Password changed, invalidating other sessions"
        );

        invalidate_other_sessions(&state.db, &state.session_store, user.id, &session_id).await?;

        // Clear user from auth cache so other sessions can't use cached auth
        auth.cache_clear_user(user.id);
        tracing::info!(user_id = user.id, "Cleared user from auth cache");
    }
    Ok(())
}

#[get("/api/realm", realm: crate::Realm)]
pub async fn get_realm() -> Result<crate::Realm> {
    Ok(realm)
}

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
        .await
        .map_err(|e| AuthError::from(anyhow::Error::from(e)))?
        .ok_or(AuthError::InvalidCredentials)?;

        let email_address_id = get_email_address_id(&state.db, user_id).await?;

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
        auth.cache_clear_user(user_id);

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
        .await
        .map_err(|e| AuthError::from(anyhow::Error::from(e)))?
        .ok_or(AuthError::InvalidCredentials)?;

        let email_address_id = get_email_address_id(&state.db, user_id).await?;

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
