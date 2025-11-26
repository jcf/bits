use bits_app::{setup_session_store, AppState, Config, User};
use axum_session::SessionLayer;
use axum_session_auth::{AuthConfig, AuthSessionLayer};
use axum_session_sqlx::SessionPgPool;
use dioxus::server::axum;

fn App() -> dioxus::prelude::Element {
    use dioxus::prelude::*;

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("assets/app.css") }
        bits_app::App {}
    }
}

/// Single-tenant server setup (solo mode)
pub async fn server(config: Config) -> Result<axum::Router, anyhow::Error> {
    use dioxus::server::axum::Extension;

    let state = bits_app::init(config.clone()).await?;
    let session_store = setup_session_store(&state).await?;

    let auth_config = AuthConfig::<i64>::default().with_anonymous_user_id(Some(-1));
    let auth_layer =
        AuthSessionLayer::<User, i64, SessionPgPool, sqlx::PgPool>::new(Some(state.db.clone()))
            .with_config(auth_config);

    Ok(dioxus::server::router(App)
        .layer(auth_layer)
        .layer(SessionLayer::new(session_store))
        .layer(Extension(config))
        .layer(Extension(state)))
}
