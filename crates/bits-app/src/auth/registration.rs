//! User registration.

#[cfg(feature = "server")]
use super::core::AuthSession;
use super::core::{AuthError, User};
#[cfg(feature = "server")]
use bits_domain::{Email, EmailAddressId, Password, PasswordHash, UserId};
use dioxus::prelude::*;
#[cfg(feature = "server")]
use dioxus::server::axum::extract::Extension;
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use sqlx::PgPool;

#[cfg(feature = "server")]
use crate::metrics::{RecordMetrics, RegisterAttempt};

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

#[cfg(feature = "server")]
async fn create_user_with_email(
    db: &PgPool,
    email: &Email,
    password_hash: &PasswordHash,
) -> anyhow::Result<UserId> {
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

    Ok(UserId::new(user_id))
}

#[cfg(feature = "server")]
pub(super) async fn get_email_address_id(
    db: &PgPool,
    user_id: UserId,
) -> anyhow::Result<EmailAddressId> {
    use anyhow::Context;

    let email_address_id = sqlx::query_scalar!(
        "select id from email_addresses where user_id = $1 and valid_to = 'infinity' limit 1",
        user_id.get()
    )
    .fetch_one(db)
    .await
    .context("Failed to get email address for user")?;

    Ok(EmailAddressId::new(email_address_id))
}

#[post("/api/users", auth: AuthSession, state: Extension<crate::AppState>)]
pub async fn join(form: dioxus::fullstack::Form<JoinForm>) -> Result<User, AuthError> {
    #[cfg(feature = "server")]
    {
        (async {
            // Parse and validate email at boundary
            let email = Email::parse(&form.0.email).map_err(|_| AuthError::InvalidCredentials)?;

            // Wrap password in domain type
            let password = Password::new(form.0.password.clone());

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
