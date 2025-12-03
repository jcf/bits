#[cfg(feature = "server")]
use axum_session_auth::{Authentication, HasPermission};
#[cfg(feature = "server")]
use dioxus::server::axum::extract::Extension;
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChangePasswordForm {
    pub current_password: String,
    pub new_password: String,
    pub confirm_password: String,
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

#[post("/api/sessions", auth: AuthSession, state: Extension<crate::AppState>)]
pub async fn auth(form: dioxus::fullstack::Form<AuthForm>) -> Result<(), AuthError> {
    #[cfg(feature = "server")]
    {
        use argon2::{PasswordHash, PasswordVerifier};

        let user = load_login_data(&state.db, &form.0.email).await?;

        let user = match user {
            Some(u) => u,
            None => {
                let email_exists = check_email_exists(&state.db, &form.0.email).await?;

                if email_exists {
                    return Err(AuthError::EmailNotVerified);
                } else {
                    return Err(AuthError::InvalidCredentials);
                }
            }
        };

        let parsed_hash =
            PasswordHash::new(&user.password_hash).map_err(|_| AuthError::InvalidCredentials)?;

        state
            .argon2
            .verify_password(form.0.password.as_bytes(), &parsed_hash)
            .map_err(|_| AuthError::InvalidCredentials)?;

        auth.session.renew();
        auth.login_user(user.user_id);
    }
    Ok(())
}

#[delete("/api/session", auth: AuthSession)]
pub async fn sign_out() -> Result<()> {
    auth.logout_user();
    Ok(())
}

#[get("/api/session", auth: AuthSession)]
pub async fn get_session() -> Result<Option<User>> {
    if let Some(user) = auth.current_user {
        if user.is_authenticated() {
            return Ok(Some(user));
        }
    }
    Ok(None)
}

#[cfg(feature = "server")]
async fn hash_password(argon2: &argon2::Argon2<'_>, password: &str) -> anyhow::Result<String> {
    use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};

    let salt = SaltString::generate(&mut OsRng);
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
async fn invalidate_all_sessions(db: &PgPool, user_id: i64) -> anyhow::Result<()> {
    use anyhow::Context;

    sqlx::query!("delete from sessions where user_id = $1", user_id)
        .execute(db)
        .await
        .context("Failed to invalidate user sessions")?;

    Ok(())
}

#[post("/api/users", state: Extension<crate::AppState>)]
pub async fn join(form: dioxus::fullstack::Form<JoinForm>) -> Result<(), AuthError> {
    #[cfg(feature = "server")]
    {
        let password_hash = hash_password(&state.argon2, &form.0.password).await?;

        match create_user_with_email(&state.db, &form.0.email, &password_hash).await {
            Ok(_) => Ok(()),
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
    #[cfg(not(feature = "server"))]
    Ok(())
}

// TODO Use #[patch] when Dioxus next ships
//
// https://github.com/DioxusLabs/dioxus/commit/57e3543c6475b5f6af066774d2152a6dd6351196
#[post("/api/passwords", auth: AuthSession, state: Extension<crate::AppState>)]
pub async fn change_password(
    form: dioxus::fullstack::Form<ChangePasswordForm>,
) -> Result<(), AuthError> {
    #[cfg(feature = "server")]
    {
        use argon2::{PasswordHash, PasswordVerifier};

        let user = auth
            .current_user
            .as_ref()
            .filter(|u| u.is_authenticated())
            .ok_or(AuthError::InvalidCredentials)?;

        if form.0.new_password != form.0.confirm_password {
            return Err(AuthError::Internal("Passwords do not match".to_string()));
        }

        let login_data = load_login_data(&state.db, &user.email).await?;
        let login_data = login_data.ok_or(AuthError::InvalidCredentials)?;

        let parsed_hash = PasswordHash::new(&login_data.password_hash)
            .map_err(|_| AuthError::InvalidCredentials)?;

        state
            .argon2
            .verify_password(form.0.current_password.as_bytes(), &parsed_hash)
            .map_err(|_| AuthError::InvalidCredentials)?;

        let new_hash = hash_password(&state.argon2, &form.0.new_password).await?;
        update_password_hash(&state.db, user.id, &new_hash).await?;
        invalidate_all_sessions(&state.db, user.id).await?;

        auth.logout_user();
    }
    Ok(())
}

#[get("/api/realm", realm: crate::Realm)]
pub async fn get_realm() -> Result<crate::Realm> {
    Ok(realm)
}
