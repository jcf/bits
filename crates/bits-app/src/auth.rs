//! Authentication and authorization.
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
//!
//! ## Endpoints by Authorization Level
//!
//! - **Public**: No authentication required
//!   - `get_realm` - Get current realm (tenant/platform/demo)
//!   - `check_subdomain` - Check subdomain availability (public demo feature)
//!
//! - **Session Management**: Uses `AuthSession`
//!   - `auth` - Sign in
//!   - `join` - Sign up
//!   - `sign_out` - Sign out
//!   - `get_session` - Check if request is authenticated
//!   - `verify_email_code` - Verify email with code (needs cache management)
//!   - `resend_verification_code` - Resend verification code
//!
//! - **Verified**: Uses `Verified` extractor
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
use crate::metrics::{LoginAttempt, LogoutAttempt, RecordMetrics, RegisterAttempt};

#[cfg(feature = "server")]
pub type AuthSession = axum_session_auth::AuthSession<
    User,
    bits_domain::UserId,
    bits_axum_session_sqlx::SessionPgPool,
    PgPool,
>;

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
    pub id: bits_domain::UserId,
    pub email: bits_domain::Email,
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
impl Authentication<User, bits_domain::UserId, PgPool> for User {
    async fn load_user(
        userid: bits_domain::UserId,
        pool: Option<&PgPool>,
    ) -> Result<User, anyhow::Error> {
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

#[derive(Clone, Serialize, Deserialize)]
pub struct JoinForm {
    pub email: String,    // Raw string from form, validated in handler
    pub password: String, // Raw string from form
}

// Custom Debug to avoid logging passwords
impl std::fmt::Debug for JoinForm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JoinForm")
            .field("email", &self.email)
            .field("password", &"<redacted>")
            .finish()
    }
}

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AuthResponse {
    Success(User),
    NeedsVerification { email: String },
}

#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
struct LoginData {
    user_id: bits_domain::UserId,
    password_hash: bits_domain::PasswordHash,
    verified: bool,
}

#[cfg(feature = "server")]
async fn load_login_data(
    db: &PgPool,
    email: &bits_domain::Email,
) -> anyhow::Result<Option<LoginData>> {
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
        let email =
            bits_domain::Email::parse(&form.0.email).map_err(|_| AuthError::InvalidCredentials)?;

        // Wrap password in domain type
        let password = bits_domain::Password::new(form.0.password.clone());

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

#[cfg(feature = "server")]
async fn create_user_with_email(
    db: &PgPool,
    email: &bits_domain::Email,
    password_hash: &bits_domain::PasswordHash,
) -> anyhow::Result<bits_domain::UserId> {
    use anyhow::Context;
    use secrecy::ExposeSecret;

    let mut tx = db.begin().await.context("Failed to begin transaction")?;

    let user_id: i64 =
        sqlx::query_scalar("insert into users (password_hash) values ($1) returning id")
            .bind(password_hash.expose_secret())
            .fetch_one(&mut *tx)
            .await
            .context("Failed to insert user")?;

    sqlx::query("insert into email_addresses (user_id, address) values ($1, $2)")
        .bind(user_id)
        .bind(email.as_str())
        .execute(&mut *tx)
        .await
        .context("Failed to insert email address")?;

    tx.commit().await.context("Failed to commit transaction")?;

    Ok(bits_domain::UserId::new(user_id))
}

#[cfg(feature = "server")]
async fn update_password_hash(
    db: &PgPool,
    user_id: bits_domain::UserId,
    password_hash: &bits_domain::PasswordHash,
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
async fn get_email_address_id(
    db: &PgPool,
    user_id: bits_domain::UserId,
) -> anyhow::Result<bits_domain::EmailAddressId> {
    use anyhow::Context;

    let email_address_id = sqlx::query_scalar!(
        "select id from email_addresses where user_id = $1 and valid_to = 'infinity' limit 1",
        user_id.get()
    )
    .fetch_one(db)
    .await
    .context("Failed to get email address for user")?;

    Ok(bits_domain::EmailAddressId::new(email_address_id))
}

#[cfg(feature = "server")]
#[allow(dead_code)]
async fn is_email_verified(db: &PgPool, user_id: bits_domain::UserId) -> anyhow::Result<bool> {
    use anyhow::Context;

    let verified = sqlx::query_scalar!(
        "select exists(
            select 1
            from email_addresses ea
            join email_verifications ev on ev.email_address_id = ea.id
            where ea.user_id = $1 and ea.valid_to = 'infinity'
        )",
        user_id.get()
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
    user_id: bits_domain::UserId,
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

#[post("/api/users", auth: AuthSession, state: Extension<crate::AppState>)]
pub async fn join(form: dioxus::fullstack::Form<JoinForm>) -> Result<User, AuthError> {
    #[cfg(feature = "server")]
    {
        (async {
            // Parse and validate email at boundary
            let email = bits_domain::Email::parse(&form.0.email)
                .map_err(|_| AuthError::InvalidCredentials)?;

            // Wrap password in domain type
            let password = bits_domain::Password::new(form.0.password.clone());

            let password_hash = state
                .password_service
                .hash_password(&password)
                .map_err(|e| AuthError::Internal(format!("Failed to hash password: {}", e)))?;

            match create_user_with_email(&state.db, &email, &password_hash).await {
                Ok(user_id) => {
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
                                        user_id = %user_id,
                                        email = %email,
                                        code = %code,
                                        "User registered, verification code generated"
                                    );
                                }
                                Err(e) => {
                                    tracing::error!(
                                        user_id = %user_id,
                                        email = %email,
                                        error = %e,
                                        "Failed to generate verification code"
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!(
                                user_id = %user_id,
                                email = %email,
                                error = %e,
                                "Failed to get email address id"
                            );
                        }
                    }

                    Ok(User {
                        id: user_id,
                        email,
                        verified: false,
                    })
                }
                Err(e) => {
                    if let Some(source) = e.source() {
                        if let Some(db_err) = source.downcast_ref::<sqlx::Error>() {
                            if let Some(pg_err) = db_err.as_database_error() {
                                if pg_err.code().as_deref() == Some("23P01") {
                                    return Err(AuthError::EmailAlreadyRegistered);
                                }
                            }
                        }
                    }
                    Err(AuthError::Internal(format!("{:#}", e)))
                }
            }
        }
        .await)
            .record(RegisterAttempt)
    }
    #[cfg(not(feature = "server"))]
    {
        // Client-side stub (never actually called)
        let _ = form;
        let _ = auth;
        let _ = state;
        unreachable!("join is server-only")
    }
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
        let current_password = bits_domain::Password::new(form.0.current_password.clone());
        let new_password = bits_domain::Password::new(form.0.new_password.clone());

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

#[get("/api/realm", realm: crate::Realm)]
pub async fn get_realm() -> Result<crate::Realm> {
    if matches!(realm, crate::Realm::NotFound) {
        dioxus::fullstack::FullstackContext::commit_http_status(
            dioxus::fullstack::StatusCode::NOT_FOUND,
            None,
        );
    }
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
        .await?
        .ok_or(AuthError::InvalidCredentials)?;

        let email_address_id = get_email_address_id(&state.db, user_id.into())
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

        let email_address_id = get_email_address_id(&state.db, user_id.into())
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
