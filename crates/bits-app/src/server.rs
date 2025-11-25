use crate::AppState;
use axum_session::{SessionConfig, SessionStore};
use axum_session_sqlx::SessionPgPool;

/// Setup session store for use in server configurations
/// Returns the session store configured to use PostgreSQL
pub async fn setup_session_store(
    state: &AppState,
) -> Result<SessionStore<SessionPgPool>, anyhow::Error> {
    let session_config = SessionConfig::default().with_table_name("sessions");
    let session_store =
        SessionStore::<SessionPgPool>::new(Some(state.db.clone().into()), session_config).await?;
    Ok(session_store)
}
