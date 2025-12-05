use dioxus::prelude::*;

#[cfg(feature = "server")]
use dioxus::fullstack::FullstackContext;

// Import page components for routing
use crate::pages::{Auth, Home, Join, Layout, VerifyEmail};

#[cfg(feature = "server")]
#[derive(Clone)]
pub struct AppState {
    pub config: std::sync::Arc<crate::Config>,
    pub db: sqlx::PgPool,
    pub password_service: crate::password::PasswordService,
    pub email_verification: crate::verification::EmailVerificationService,
    pub auth_rate_limit: crate::auth_rate_limit::AuthRateLimitService,
    pub metrics_handle: metrics_exporter_prometheus::PrometheusHandle,
    pub session_store:
        std::sync::Arc<tokio::sync::Mutex<bits_axum_session_sqlx::SessionPgSessionStore>>,
}

#[cfg(feature = "server")]
impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("config", &self.config)
            .field("db", &self.db)
            .field("password_service", &"<PasswordService>")
            .field("email_verification", &"<EmailVerificationService>")
            .field("auth_rate_limit", &"<AuthRateLimitService>")
            .field("metrics_handle", &"<PrometheusHandle>")
            .field("session_store", &"<SessionStore>")
            .finish()
    }
}

#[cfg(feature = "server")]
impl AppState {
    pub async fn new(config: crate::Config) -> Result<Self, anyhow::Error> {
        let db = sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.max_database_connections)
            .connect(config.database_url.as_ref())
            .await?;

        // Initialize password service with Argon2id
        let password_service = crate::password::PasswordService::new(&config)?;

        // Derive email verification key from master key using HKDF
        use hkdf::Hkdf;
        use sha2::Sha256;
        let hk = Hkdf::<Sha256>::new(None, config.master_key.as_bytes());
        let mut verification_key = [0u8; 32];
        hk.expand(b"email-verification-v1", &mut verification_key)
            .expect("32 bytes is valid for HKDF-SHA256");

        // Initialize email verification service with derived key
        let email_verification = crate::verification::EmailVerificationService::new(
            crate::verification::EmailVerificationConfig::default(),
            verification_key.to_vec(),
        );

        // Initialize auth rate limiting service
        let auth_rate_limit = crate::auth_rate_limit::AuthRateLimitService::new();

        // Initialize metrics
        let metrics_handle = crate::metrics::init();

        // Initialize session store
        let session_config = axum_session::SessionConfig::default()
            .with_session_name(config.session_name.clone())
            .with_table_name("sessions")
            .with_secure(true)
            .with_http_only(true)
            .with_cookie_same_site(cookie::SameSite::Strict);
        let session_store =
            axum_session::SessionStore::<bits_axum_session_sqlx::SessionPgPool>::new(
                Some(db.clone().into()),
                session_config,
            )
            .await?;

        Ok(Self {
            config: std::sync::Arc::new(config),
            db,
            password_service,
            email_verification,
            auth_rate_limit,
            metrics_handle,
            session_store: std::sync::Arc::new(tokio::sync::Mutex::new(session_store)),
        })
    }
}

#[cfg(feature = "server")]
impl axum_core::extract::FromRef<FullstackContext> for AppState {
    fn from_ref(state: &FullstackContext) -> Self {
        state.extension::<AppState>().unwrap()
    }
}

#[cfg(target_arch = "wasm32")]
pub fn init_client() {
    use ::http::{HeaderMap, HeaderName, HeaderValue};

    let version = option_env!("BITS_VERSION").unwrap_or("dev");
    let header_value = format!("bits/{}", version);

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("requested-with"),
        HeaderValue::from_str(&header_value).unwrap(),
    );

    // Read CSRF token from meta tag
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(meta) = document
                .query_selector("meta[name='csrf-token']")
                .ok()
                .flatten()
            {
                if let Some(token) = meta.get_attribute("content") {
                    if !token.is_empty() {
                        if let Ok(value) = HeaderValue::from_str(&token) {
                            headers.insert(HeaderName::from_static("csrf-token"), value);
                        }
                    }
                }
            }
        }
    }

    dioxus_fullstack::set_request_headers(headers);
}

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Layout)]
    #[route("/")]
    Home {},
    #[route("/auth")]
    Auth {},
    #[route("/join")]
    Join {},
    #[route("/verify-email")]
    VerifyEmail {},
}

#[component]
pub fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: "data:" }
        Router::<Route> {}
    }
}
