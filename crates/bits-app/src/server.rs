use crate::{AppState, Config, CspMode, CsrfLayer, User};
use axum::http::{header, HeaderValue, StatusCode};
use axum_session::{SessionConfig, SessionLayer, SessionStore};
use axum_session_auth::{AuthConfig, AuthSessionLayer};
use axum_session_sqlx::SessionPgPool;
use dioxus::prelude::Element;
use dioxus::server::axum::{self, Extension};
use sqlx::PgPool;
use std::time::Duration;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::timeout::TimeoutLayer;

#[cfg(feature = "colo")]
use crate::RealmLayer;

/// Initialize tracing with custom filters for database and session logging
pub fn init_tracing() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            let filters = [
                "info",
                "axum_session=warn",
                "axum_session_auth=warn",
                "axum_session_sqlx=warn",
                "bits_app=debug",
                "bits_colo=debug",
                "bits_solo=debug",
                "sqlx=warn",
            ];
            EnvFilter::new(filters.join(","))
        }))
        .with(tracing_subscriber::fmt::layer().compact())
        .init();
}

/// Initialize the application: database and migrations
pub async fn init(config: Config) -> Result<AppState, anyhow::Error> {
    let state = AppState::new(config).await?;
    run_migrations(&state.db).await?;

    Ok(state)
}

async fn run_migrations(pool: &PgPool) -> Result<(), anyhow::Error> {
    sqlx::migrate!("../../migrations").run(pool).await?;
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

/// Build a production-ready router with security middleware
pub async fn router(config: Config, app: fn() -> Element) -> Result<axum::Router, anyhow::Error> {
    let state = init(config.clone()).await?;
    let session_store = setup_session_store(&state).await?;

    let auth_config = AuthConfig::<i64>::default().with_anonymous_user_id(Some(-1));
    let auth_layer =
        AuthSessionLayer::<User, i64, SessionPgPool, sqlx::PgPool>::new(Some(state.db.clone()))
            .with_config(auth_config);

    let csp_mode = if config.dangerously_allow_javascript_evaluation {
        CspMode::Development
    } else {
        CspMode::Strict
    };
    let csp = crate::http::csp_header(csp_mode);

    #[allow(unused_mut)]
    let mut router = dioxus::server::router(app);

    #[cfg(feature = "colo")]
    {
        router = router.layer(RealmLayer);
    }

    Ok(router
        .layer(CsrfLayer)
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("content-security-policy"),
            HeaderValue::try_from(csp).unwrap(),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("strict-origin"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::SERVER,
            HeaderValue::from_static("bits"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("strict-transport-security"),
            HeaderValue::from_static("max-age=31536000; includeSubdomains"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("x-download-options"),
            HeaderValue::from_static("noopen"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("x-permitted-cross-domain-policies"),
            HeaderValue::from_static("none"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("x-xss-protection"),
            HeaderValue::from_static("1; mode=block"),
        ))
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(auth_layer)
        .layer(SessionLayer::new(session_store))
        .layer(Extension(config))
        .layer(Extension(state)))
}
