// Module declarations
pub mod app;
pub mod auth;
pub mod components;
pub mod config;
pub mod http;
pub mod i18n;
pub mod pages;
pub mod tenant;

#[cfg(feature = "server")]
pub mod crypto;
#[cfg(feature = "server")]
pub mod csrf;
#[cfg(feature = "server")]
pub mod metrics;
#[cfg(feature = "server")]
pub mod middleware;
#[cfg(feature = "server")]
pub mod server;
#[cfg(feature = "server")]
pub mod verification;

// Re-exports - Core app
#[cfg(target_arch = "wasm32")]
pub use app::init_client;
pub use app::{App, Route};

#[cfg(feature = "server")]
pub use app::AppState;

// Re-exports - Auth
pub use auth::{
    AuthError, AuthForm, AuthResponse, ChangePasswordForm, JoinForm, ResendForm, ResendResponse,
    SessionState, User, VerifyEmailForm,
};

// Re-exports - Config
pub use config::Config;

// Re-exports - HTTP
pub use http::CspMode;

// Re-exports - Tenant
pub use tenant::{Realm, Tenant};

// Re-exports - Server middleware
#[cfg(feature = "server")]
pub use middleware::{
    CsrfVerificationLayer, CsrfVerificationMiddleware, RealmLayer, RealmMiddleware,
};

// Re-exports - Server functions
#[cfg(feature = "server")]
pub use server::{build_router, init, init_tracing, setup_session_store};
