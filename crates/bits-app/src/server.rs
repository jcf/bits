use crate::{AppState, Config};
use axum_session::{SessionConfig, SessionStore};
use axum_session_sqlx::SessionPgPool;
use sqlx::PgPool;

/// Initialize the application: tracing, database, and migrations
pub async fn init(config: Config) -> Result<AppState, anyhow::Error> {
    init_tracing();

    let state = AppState::new(config).await?;
    run_migrations(&state.db).await?;

    Ok(state)
}

fn init_tracing() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                EnvFilter::new(
                    "info,sqlx=warn,axum_session=warn,axum_session_auth=warn,axum_session_sqlx=warn",
                )
            }),
        )
        .with(tracing_subscriber::fmt::layer().compact())
        .init();
}

async fn run_migrations(pool: &PgPool) -> Result<(), anyhow::Error> {
    sqlx::migrate!().run(pool).await?;
    Ok(())
}

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
