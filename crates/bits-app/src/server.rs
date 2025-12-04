use crate::{AppState, Config, CspMode, CsrfVerificationLayer, User};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum_session::{SessionConfig, SessionLayer, SessionStore};
use axum_session_auth::{AuthConfig, AuthSessionLayer};
use bits_axum_session_sqlx::SessionPgPool;
use cookie::SameSite;
use dioxus::prelude::Element;
use dioxus::server::axum::{self, Extension};
use sqlx::PgPool;
use std::time::Duration;
use tower_http::cors::CorsLayer;
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
    let session_config = SessionConfig::default()
        .with_session_name(state.config.session_name.clone())
        .with_table_name("sessions")
        .with_secure(true)
        .with_http_only(true)
        .with_cookie_same_site(SameSite::Strict);
    let session_store =
        SessionStore::<SessionPgPool>::new(Some(state.db.clone().into()), session_config).await?;
    Ok(session_store)
}

/// Metrics endpoint handler with optional bearer token authentication
async fn metrics_handler(
    Extension(state): Extension<AppState>,
    headers: axum::http::HeaderMap,
) -> Response {
    // Check if auth token is configured
    if let Some(required_token) = &state.config.metrics_auth_token {
        // Extract bearer token from Authorization header
        let auth_header = headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok());

        let provided_token = auth_header.and_then(|h| h.strip_prefix("Bearer "));

        // Compare tokens using constant-time comparison to prevent timing attacks
        use subtle::ConstantTimeEq;
        let authorized = provided_token
            .map(|token| token.as_bytes().ct_eq(required_token.as_bytes()).into())
            .unwrap_or(false);

        if !authorized {
            return (
                StatusCode::UNAUTHORIZED,
                "Unauthorized: Invalid or missing bearer token",
            )
                .into_response();
        }
    }

    state.metrics_handle.render().into_response()
}

/// Build a production-ready router with security middleware
pub async fn build_router(
    state: AppState,
    app: fn() -> Element,
) -> Result<axum::Router, anyhow::Error> {
    let session_store = state.session_store.lock().await.clone();

    let auth_config = AuthConfig::<i64>::default().with_anonymous_user_id(Some(-1));

    let auth_layer =
        AuthSessionLayer::<User, i64, SessionPgPool, sqlx::PgPool>::new(Some(state.db.clone()))
            .with_config(auth_config);

    let csp_mode = if state.config.dangerously_allow_javascript_evaluation {
        CspMode::Development
    } else {
        CspMode::Strict
    };
    let csp = crate::http::csp_header(csp_mode);

    #[allow(unused_mut)]
    let mut router = dioxus::server::router(app);

    // Add metrics endpoint
    router = router.route("/metrics", axum::routing::get(metrics_handler));

    #[cfg(feature = "colo")]
    {
        router = router.layer(RealmLayer);
    }

    // Explicitly reject all cross-origin requests
    // Each tenant accesses their own domain, no cross-origin requests needed
    let cors = CorsLayer::new()
        .allow_methods(vec![])
        .allow_headers(vec![])
        .allow_origin(tower_http::cors::AllowOrigin::predicate(|_, _| false));

    let router = router
        .layer(axum::middleware::from_fn(
            crate::middleware::metrics::track_metrics,
        ))
        .layer(cors)
        .layer(CsrfVerificationLayer)
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
        .layer(Extension(state.config.clone()))
        .layer(Extension(state));

    Ok(router)
}
