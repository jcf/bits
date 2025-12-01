use axum::http::{header, HeaderValue};
use axum_session::SessionLayer;
use axum_session_auth::{AuthConfig, AuthSessionLayer};
use axum_session_sqlx::SessionPgPool;
use bits_app::{setup_session_store, Config, CspMode, User};
use dioxus::server::axum;
use tower_http::set_header::SetResponseHeaderLayer;

#[allow(non_snake_case)]
fn App() -> dioxus::prelude::Element {
    use dioxus::prelude::*;

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("assets/app.css") }
        bits_app::App {}
    }
}

/// Build router for single-tenant mode (solo)
pub async fn router(config: Config) -> Result<axum::Router, anyhow::Error> {
    use dioxus::server::axum::Extension;

    let state = bits_app::init(config.clone()).await?;
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
    let csp = bits_app::http::csp_header(csp_mode);

    Ok(dioxus::server::router(App)
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
        .layer(auth_layer)
        .layer(SessionLayer::new(session_store))
        .layer(Extension(config))
        .layer(Extension(state)))
}
