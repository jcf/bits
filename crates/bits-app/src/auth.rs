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

        let user = sqlx::query_as::<_, User>(
            "select user_id as id, email from logins where user_id = $1 limit 1",
        )
        .bind(userid)
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

#[post("/api/auth")]
pub async fn auth(form: dioxus::fullstack::Form<AuthForm>) -> Result<(), AuthError> {
    #[cfg(feature = "server")]
    {
        use crate::AppState;
        use argon2::{Argon2, PasswordHash, PasswordVerifier};
        use dioxus::fullstack::FullstackContext;
        use dioxus::server::axum::extract::Extension;

        let auth = FullstackContext::extract::<AuthSession, _>()
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;
        let Extension(state) = FullstackContext::extract::<Extension<AppState>, _>()
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        #[derive(sqlx::FromRow)]
        struct LoginData {
            user_id: i64,
            password_hash: String,
        }

        let user = sqlx::query_as::<_, LoginData>(
            "select user_id, password_hash from logins where email = $1 limit 1",
        )
        .bind(&form.0.email)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AuthError::Internal(e.to_string()))?;

        let user = match user {
            Some(u) => u,
            None => {
                let email_exists: Option<(i64,)> = sqlx::query_as(
                    "select user_id from email_addresses where address = $1 and valid_to = 'infinity' limit 1",
                )
                .bind(&form.0.email)
                .fetch_optional(&state.db)
                .await
                .map_err(|e| AuthError::Internal(e.to_string()))?;

                if email_exists.is_some() {
                    return Err(AuthError::EmailNotVerified);
                } else {
                    return Err(AuthError::InvalidCredentials);
                }
            }
        };

        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        if Argon2::default()
            .verify_password(form.0.password.as_bytes(), &parsed_hash)
            .is_err()
        {
            return Err(AuthError::InvalidCredentials);
        }

        auth.login_user(user.user_id);
    }
    Ok(())
}

#[post("/api/sign-out")]
pub async fn sign_out() -> Result<()> {
    #[cfg(feature = "server")]
    {
        use dioxus::fullstack::FullstackContext;

        let auth = FullstackContext::extract::<AuthSession, _>().await?;
        auth.logout_user();
    }
    Ok(())
}

#[get("/api/session")]
pub async fn get_session() -> Result<Option<User>> {
    #[cfg(feature = "server")]
    {
        use dioxus::fullstack::FullstackContext;

        let auth = FullstackContext::extract::<AuthSession, _>().await?;

        if let Some(user) = auth.current_user {
            if user.is_authenticated() {
                return Ok(Some(user));
            }
        }
    }
    Ok(None)
}

#[post("/api/join")]
pub async fn join(form: dioxus::fullstack::Form<JoinForm>) -> Result<(), AuthError> {
    #[cfg(feature = "server")]
    {
        use crate::AppState;
        use argon2::{
            password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
            Argon2,
        };
        use dioxus::fullstack::FullstackContext;
        use dioxus::server::axum::extract::Extension;

        let Extension(state) = FullstackContext::extract::<Extension<AppState>, _>()
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(form.0.password.as_bytes(), &salt)
            .map_err(|e| AuthError::Internal(e.to_string()))?
            .to_string();

        let mut tx = state
            .db
            .begin()
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        let user_id: i64 =
            sqlx::query_scalar("insert into users (password_hash) values ($1) returning id")
                .bind(&password_hash)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| AuthError::Internal(e.to_string()))?;

        match sqlx::query("insert into email_addresses (user_id, address) values ($1, $2)")
            .bind(user_id)
            .bind(&form.0.email)
            .execute(&mut *tx)
            .await
        {
            Ok(_) => {
                tx.commit()
                    .await
                    .map_err(|e| AuthError::Internal(e.to_string()))?;
                Ok(())
            }
            Err(e) => {
                tx.rollback()
                    .await
                    .map_err(|e| AuthError::Internal(e.to_string()))?;
                if let Some(db_err) = e.as_database_error() {
                    if db_err.code().as_deref() == Some("23P01") {
                        return Err(AuthError::EmailAlreadyRegistered);
                    }
                }
                Err(AuthError::Internal(e.to_string()))
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
