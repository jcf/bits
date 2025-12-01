#[cfg(feature = "server")]
use axum_session_auth::{Authentication, HasPermission};
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use sqlx::PgPool;

use dioxus::prelude::*;

#[cfg(feature = "server")]
pub type AuthSession =
    axum_session_auth::AuthSession<User, i64, axum_session_sqlx::SessionPgPool, PgPool>;

#[derive(thiserror::Error, Debug, Serialize, Deserialize)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Email not verified")]
    EmailNotVerified,

    #[error("Email already registered")]
    EmailAlreadyRegistered,

    #[error("Internal error")]
    Internal(String),
}

impl From<ServerFnError> for AuthError {
    fn from(err: ServerFnError) -> Self {
        AuthError::Internal(err.to_string())
    }
}

impl dioxus::fullstack::AsStatusCode for AuthError {
    fn as_status_code(&self) -> dioxus::fullstack::StatusCode {
        match self {
            AuthError::InvalidCredentials => dioxus::fullstack::StatusCode::UNAUTHORIZED,
            AuthError::EmailNotVerified => dioxus::fullstack::StatusCode::FORBIDDEN,
            AuthError::EmailAlreadyRegistered => dioxus::fullstack::StatusCode::BAD_REQUEST,
            AuthError::Internal(_) => dioxus::fullstack::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub email: String,
}

#[cfg(feature = "server")]
impl Default for User {
    fn default() -> Self {
        Self {
            id: -1,
            email: String::new(),
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
            "select user_id as \"id!\", email as \"email!\" from logins where user_id = $1 limit 1",
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

#[cfg(feature = "server")]
async fn extract_auth_session() -> anyhow::Result<AuthSession> {
    use anyhow::Context;
    use dioxus::fullstack::FullstackContext;

    FullstackContext::extract::<AuthSession, _>()
        .await
        .context("Failed to extract auth session from request")
}

#[cfg(feature = "server")]
async fn extract_app_state() -> anyhow::Result<crate::AppState> {
    use anyhow::Context;
    use dioxus::fullstack::FullstackContext;
    use dioxus::server::axum::extract::Extension;

    let Extension(state) = FullstackContext::extract::<Extension<crate::AppState>, _>()
        .await
        .context("Failed to extract app state from request")?;
    Ok(state)
}

#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
struct LoginData {
    user_id: i64,
    password_hash: String,
}

#[cfg(feature = "server")]
async fn load_login_data(db: &PgPool, email: &str) -> anyhow::Result<Option<LoginData>> {
    use anyhow::Context;

    sqlx::query_as!(
        LoginData,
        "select user_id as \"user_id!\", password_hash as \"password_hash!\" from logins where email = $1 limit 1",
        email
    )
    .fetch_optional(db)
    .await
    .context("Failed to query login data")
}

#[cfg(feature = "server")]
async fn check_email_exists(db: &PgPool, email: &str) -> anyhow::Result<bool> {
    use anyhow::Context;

    let exists: Option<i64> = sqlx::query_scalar!(
        "select user_id from email_addresses where address = $1 and valid_to = 'infinity' limit 1",
        email
    )
    .fetch_optional(db)
    .await
    .context("Failed to check if email exists")?;

    Ok(exists.is_some())
}

#[post("/auth")]
pub async fn auth(form: dioxus::fullstack::Form<AuthForm>) -> Result<(), AuthError> {
    #[cfg(feature = "server")]
    {
        use argon2::{Argon2, PasswordHash, PasswordVerifier};

        let auth = extract_auth_session()
            .await
            .map_err(|e| AuthError::Internal(format!("{:#}", e)))?;
        let state = extract_app_state()
            .await
            .map_err(|e| AuthError::Internal(format!("{:#}", e)))?;

        let user = load_login_data(&state.db, &form.0.email)
            .await
            .map_err(|e| AuthError::Internal(format!("{:#}", e)))?;

        let user = match user {
            Some(u) => u,
            None => {
                let email_exists = check_email_exists(&state.db, &form.0.email)
                    .await
                    .map_err(|e| AuthError::Internal(format!("{:#}", e)))?;

                if email_exists {
                    return Err(AuthError::EmailNotVerified);
                } else {
                    return Err(AuthError::InvalidCredentials);
                }
            }
        };

        // Password verification - any error should return InvalidCredentials for security
        let parsed_hash =
            PasswordHash::new(&user.password_hash).map_err(|_| AuthError::InvalidCredentials)?;

        Argon2::default()
            .verify_password(form.0.password.as_bytes(), &parsed_hash)
            .map_err(|_| AuthError::InvalidCredentials)?;

        auth.login_user(user.user_id);
    }
    Ok(())
}

#[post("/bye")]
pub async fn sign_out() -> Result<()> {
    use dioxus::fullstack::FullstackContext;
    let auth = FullstackContext::extract::<AuthSession, _>().await?;
    auth.logout_user();
    Ok(())
}

#[get("/api/session")]
pub async fn get_session() -> Result<Option<User>> {
    use dioxus::fullstack::FullstackContext;

    let auth = FullstackContext::extract::<AuthSession, _>().await?;

    if let Some(user) = auth.current_user {
        if user.is_authenticated() {
            return Ok(Some(user));
        }
    }
    Ok(None)
}

#[cfg(feature = "server")]
async fn hash_password(password: &str) -> anyhow::Result<String> {
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?
        .to_string();

    Ok(hash)
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

#[post("/join")]
pub async fn join(form: dioxus::fullstack::Form<JoinForm>) -> Result<(), AuthError> {
    #[cfg(feature = "server")]
    {
        let state = extract_app_state()
            .await
            .map_err(|e| AuthError::Internal(format!("{:#}", e)))?;

        let password_hash = hash_password(&form.0.password)
            .await
            .map_err(|e| AuthError::Internal(format!("{:#}", e)))?;

        match create_user_with_email(&state.db, &form.0.email, &password_hash).await {
            Ok(_) => Ok(()),
            Err(e) => {
                // Check for exclusion constraint violation on email_addresses table
                if let Some(source) = e.source() {
                    if let Some(db_err) = source.downcast_ref::<sqlx::Error>() {
                        if let Some(pg_err) = db_err.as_database_error() {
                            // 23P01 = exclusion_violation (temporal unique constraint on email)
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
    #[cfg(not(feature = "server"))]
    Ok(())
}

#[get("/api/realm", realm: crate::Realm)]
pub async fn get_realm() -> Result<crate::Realm> {
    Ok(realm)
}
