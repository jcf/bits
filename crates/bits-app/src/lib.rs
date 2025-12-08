#![forbid(unsafe_code)]
#![deny(clippy::fallible_impl_from)]
#![deny(clippy::fn_params_excessive_bools)]
#![deny(clippy::indexing_slicing)]
#![deny(clippy::must_use_candidate)]
#![deny(clippy::unneeded_field_pattern)]
#![deny(clippy::wildcard_enum_match_arm)]

// Module declarations
pub mod app;
pub mod auth;
pub mod components;
pub mod config;
pub mod http;
pub mod i18n;
pub mod pages;
pub mod subdomain;
pub mod tenant;

#[cfg(feature = "server")]
pub mod auth_rate_limit;
#[cfg(feature = "server")]
pub mod csrf;
pub mod demos;
#[cfg(feature = "server")]
pub mod metrics;
#[cfg(feature = "server")]
pub mod middleware;
#[cfg(feature = "server")]
pub mod password;
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
    SessionState, User, UserState, VerifyEmailForm,
};

#[cfg(feature = "server")]
pub use auth::{Authenticated, Verified};

// Re-exports - Config
pub use config::Config;

#[cfg(feature = "server")]
pub use config::load_config;

// Re-exports - HTTP
pub use http::CspMode;

// Re-exports - Tenant
pub use tenant::{Handle, HandleError, Realm, Tenant};

// Re-exports - Server middleware
#[cfg(feature = "server")]
pub use middleware::{
    CsrfVerificationLayer, CsrfVerificationMiddleware, RealmLayer, RealmMiddleware,
};

// Re-exports - Server functions
#[cfg(feature = "server")]
pub use server::{build_router, init, init_tracing, setup_session_store};
