use crate::{AppState, Config, CspMode, CsrfVerificationLayer, User};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum_governor::GovernorLayer;
use axum_session::{SessionConfig, SessionLayer, SessionStore};
use axum_session_auth::{AuthConfig, AuthSessionLayer};
use bits_axum_session_sqlx::SessionPgPool;
use cookie::SameSite;
use dioxus::server::axum::{self, Extension};
use lazy_limit::{init_rate_limiter, RuleConfig};
use real::RealIpLayer;
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
                "dioxus_server::server=warn",
                "sqlx=warn",
            ];
            EnvFilter::new(filters.join(","))
        }))
        .with(tracing_subscriber::fmt::layer().compact())
        .init();
}

/// Initialize the application: database and migrations
pub async fn init(config: Config) -> Result<AppState, anyhow::Error> {
    if let Some(limit) = config.global_rate_limit {
        use std::sync::atomic::{AtomicBool, Ordering};
        static RATE_LIMITER_INITIALIZED: AtomicBool = AtomicBool::new(false);

        if !RATE_LIMITER_INITIALIZED.swap(true, Ordering::SeqCst) {
            init_rate_limiter!(
                default: RuleConfig::new(lazy_limit::Duration::seconds(1), limit),
                max_memory: Some(64 * 1024 * 1024)
            )
            .await;
        }
    }

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

// TODO Add health checks to core components within `AppState`.
/// Health check endpoint for monitoring and systemd checks
/// Verifies database connectivity without authentication
async fn healthz_handler(Extension(state): Extension<AppState>) -> Response {
    match sqlx::query("select 1").fetch_one(&state.db).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => {
            tracing::error!("Health check failed: {}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "Database connection failed",
            )
                .into_response()
        }
    }
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

const MAX_REQUEST_BODY_BYTES: usize = 10 * 1024 * 1024;
const REQUEST_TIMEOUT_SECONDS: u64 = 30;

fn base_router(app: fn() -> dioxus::prelude::Element) -> axum::Router {
    let mut router = dioxus::server::router(app);
    router = router.route("/healthz", axum::routing::get(healthz_handler));
    router = router.route("/metrics", axum::routing::get(metrics_handler));

    #[cfg(feature = "colo")]
    {
        router = router.layer(RealmLayer);
    }

    router
}

/// Build a production-ready router with security middleware
pub async fn build_router(
    state: AppState,
    app: fn() -> dioxus::prelude::Element,
) -> Result<axum::Router, anyhow::Error> {
    let session_store = state.session_store.lock().await.clone();

    let csp_mode = if state.config.dangerously_allow_javascript_evaluation {
        CspMode::Development
    } else {
        CspMode::Strict
    };
    let csp = crate::http::csp_header(csp_mode);

    // CORS: reject all cross-origin requests (each tenant uses their own domain)
    let cors_layer = CorsLayer::new()
        .allow_methods(vec![])
        .allow_headers(vec![])
        .allow_origin(tower_http::cors::AllowOrigin::predicate(|_, _| false));

    // Authentication session
    let auth_config = AuthConfig::<bits_domain::UserId>::default()
        .with_anonymous_user_id(Some(bits_domain::UserId::new(-1)));
    let auth_layer =
        AuthSessionLayer::<User, bits_domain::UserId, SessionPgPool, sqlx::PgPool>::new(Some(
            state.db.clone(),
        ))
        .with_config(auth_config);

    // Request tracking
    let request_id_layer = SetRequestIdLayer::x_request_id(MakeRequestUuid);
    let propagate_id_layer = PropagateRequestIdLayer::x_request_id();

    // Limits
    let body_limit_layer = RequestBodyLimitLayer::new(MAX_REQUEST_BODY_BYTES);
    let timeout_layer = TimeoutLayer::with_status_code(
        StatusCode::REQUEST_TIMEOUT,
        Duration::from_secs(REQUEST_TIMEOUT_SECONDS),
    );

    // Security headers
    let security_headers = [
        ("content-security-policy", csp.as_str()),
        ("referrer-policy", "strict-origin"),
        ("server", "bits"),
        (
            "strict-transport-security",
            "max-age=31536000; includeSubdomains",
        ),
        ("x-content-type-options", "nosniff"),
        ("x-download-options", "noopen"),
        ("x-frame-options", "DENY"),
        ("x-permitted-cross-domain-policies", "none"),
        ("x-xss-protection", "1; mode=block"),
    ];

    let mut router = base_router(app);

    // Apply middleware layers in order (outermost to innermost)
    router = router.layer(axum::middleware::from_fn(
        crate::middleware::metrics::track_metrics,
    ));
    router = router.layer(axum::middleware::from_fn_with_state(
        state.clone(),
        crate::middleware::auth_rate_limit::auth_rate_limit_middleware,
    ));
    router = router.layer(cors_layer);
    router = router.layer(CsrfVerificationLayer);

    if state.config.global_rate_limit.is_some() {
        router = router.layer(RealIpLayer::default());
        router = router.layer(GovernorLayer::default());
    }

    for (name, value) in security_headers {
        router = router.layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::try_from(name).unwrap(),
            HeaderValue::try_from(value).unwrap(),
        ));
    }

    router = router.layer(body_limit_layer);
    router = router.layer(timeout_layer);
    router = router.layer(propagate_id_layer);
    router = router.layer(request_id_layer);
    router = router.layer(auth_layer);
    router = router.layer(SessionLayer::new(session_store));
    router = router.layer(Extension(state.config.clone()));
    router = router.layer(Extension(state));

    Ok(router)
}
