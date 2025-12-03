pub mod auth;
pub mod components;
pub mod config;
pub mod http;
pub mod i18n;
pub mod tenant;

#[cfg(feature = "server")]
pub mod crypto;
#[cfg(feature = "server")]
pub mod csrf;
#[cfg(feature = "server")]
pub mod middleware;
#[cfg(feature = "server")]
pub mod server;

pub use auth::{AuthError, AuthForm, ChangePasswordForm, JoinForm, User};
pub use config::Config;
pub use http::CspMode;
pub use tenant::{Realm, Tenant};

#[cfg(feature = "server")]
pub use middleware::{
    CsrfVerificationLayer, CsrfVerificationMiddleware, RealmLayer, RealmMiddleware,
};
#[cfg(feature = "server")]
pub use server::{init, init_tracing, router, setup_session_store};

#[cfg(feature = "server")]
use dioxus::fullstack::FullstackContext;

#[cfg(feature = "server")]
#[derive(Clone, Debug)]
pub struct AppState {
    pub config: std::sync::Arc<Config>,
    pub db: sqlx::PgPool,
    pub argon2: argon2::Argon2<'static>,
    pub crypto: crypto::EncryptionService,
}

#[cfg(feature = "server")]
impl AppState {
    pub async fn new(config: Config) -> Result<Self, anyhow::Error> {
        let db = sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.max_database_connections)
            .connect(config.database_url.as_ref())
            .await?;

        // Configure Argon2id with explicit parameters per OWASP recommendations
        let argon2_params = argon2::Params::new(
            config.argon2_m_cost,
            config.argon2_t_cost,
            config.argon2_p_cost,
            Some(argon2::Params::DEFAULT_OUTPUT_LEN),
        )
        .map_err(|e| anyhow::anyhow!("Invalid Argon2 parameters: {}", e))?;

        let argon2 = argon2::Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2_params,
        );

        // Initialize encryption service from master key
        let crypto = crypto::EncryptionService::new(&config.master_key)
            .map_err(|e| anyhow::anyhow!("Failed to initialize encryption service: {}", e))?;

        Ok(Self {
            config: std::sync::Arc::new(config),
            db,
            argon2,
            crypto,
        })
    }
}

#[cfg(feature = "server")]
impl axum_core::extract::FromRef<FullstackContext> for AppState {
    fn from_ref(state: &FullstackContext) -> Self {
        state.extension::<AppState>().unwrap()
    }
}

// App module
use dioxus::prelude::*;

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
}

#[component]
pub fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: "data:" }
        Router::<Route> {}
    }
}

#[component]
fn Auth() -> Element {
    use auth::{auth, AuthForm};
    use components::Input;

    let mut session = use_context::<Resource<Result<Option<User>>>>();
    let mut auth_action = use_action(auth);
    let nav = navigator();
    let t = i18n::use_translation();

    rsx! {
        div { class: "flex min-h-full items-center justify-center px-4 py-12",
            div { class: "w-full max-w-sm space-y-10",
                h2 { class: "mt-10 text-center text-2xl font-bold text-gray-900 dark:text-white",
                    "{t.t(\"auth-sign-in-title\")}"
                }
                if let Some(Err(err)) = auth_action.value() {
                    div { class: "text-red-500 text-sm text-center", "{err}" }
                }
                form {
                    method: "post",
                    class: "space-y-6",
                    onsubmit: move |evt: FormEvent| async move {
                        evt.prevent_default();
                        let form: AuthForm = evt.parsed_values().unwrap();
                        auth_action.call(dioxus::fullstack::Form(form)).await;
                        if auth_action.value().and_then(|r| r.ok()).is_some() {
                            session.restart();
                            nav.push(Route::Home {});
                        }
                    },
                    div {
                        Input {
                            id: "email",
                            input_type: "email",
                            name: "email",
                            placeholder: t.t("form-email-placeholder"),
                        }
                    }
                    div {
                        Input {
                            id: "password",
                            input_type: "password",
                            name: "password",
                            placeholder: t.t("form-password-placeholder"),
                        }
                    }
                    components::Button {
                        button_type: "submit",
                        variant: components::ButtonVariant::Primary,
                        size: components::ButtonSize::LG,
                        loading: auth_action.pending(),
                        class: "w-full",
                        if auth_action.pending() {
                            "{t.t(\"auth-sign-in-loading\")}"
                        } else {
                            "{t.t(\"auth-sign-in-button\")}"
                        }
                    }
                }
                p { class: "text-center text-sm text-gray-500",
                    "{t.t(\"auth-not-member\")} "
                    Link {
                        to: Route::Join {},
                        class: "text-indigo-600 hover:text-indigo-500",
                        "{t.t(\"auth-create-account-link\")}"
                    }
                }
            }
        }
    }
}

#[component]
fn Join() -> Element {
    use auth::{join, JoinForm};
    use components::Input;

    let mut join_action = use_action(join);
    let nav = navigator();
    let t = i18n::use_translation();

    rsx! {
        div { class: "flex min-h-full items-center justify-center px-4 py-12",
            div { class: "w-full max-w-sm space-y-10",
                h2 { class: "mt-10 text-center text-2xl font-bold text-gray-900 dark:text-white",
                    "{t.t(\"auth-create-account-title\")}"
                }
                if let Some(Err(err)) = join_action.value() {
                    div { class: "text-red-500 text-sm text-center", "{err}" }
                }
                if let Some(Ok(_)) = join_action.value() {
                    div { class: "text-green-500 text-sm text-center",
                        "{t.t(\"auth-account-created-success\")}"
                    }
                }
                form {
                    method: "post",
                    class: "space-y-6",
                    onsubmit: move |evt: FormEvent| async move {
                        evt.prevent_default();
                        let form: JoinForm = evt.parsed_values().unwrap();
                        join_action.call(dioxus::fullstack::Form(form)).await;
                        if join_action.value().and_then(|r| r.ok()).is_some() {
                            nav.push(Route::Auth {});
                        }
                    },
                    div {
                        Input {
                            id: "email",
                            input_type: "email",
                            name: "email",
                            placeholder: t.t("form-email-placeholder"),
                        }
                    }
                    div {
                        Input {
                            id: "password",
                            input_type: "password",
                            name: "password",
                            placeholder: t.t("form-password-placeholder"),
                        }
                    }
                    components::Button {
                        button_type: "submit",
                        variant: components::ButtonVariant::Primary,
                        size: components::ButtonSize::LG,
                        loading: join_action.pending(),
                        class: "w-full",
                        if join_action.pending() {
                            "{t.t(\"auth-create-account-loading\")}"
                        } else {
                            "{t.t(\"auth-create-account-button\")}"
                        }
                    }
                }
                p { class: "text-center text-sm text-gray-500",
                    "{t.t(\"auth-already-member\")} "
                    Link {
                        to: Route::Auth {},
                        class: "text-indigo-600 hover:text-indigo-500",
                        "{t.t(\"auth-sign-in-button\")}"
                    }
                }
            }
        }
    }
}

#[component]
fn NotFound() -> Element {
    let t = i18n::use_translation();

    rsx! {
        div { class: "text-red-500",
            h1 { "{t.t(\"error-404-title\")}" }
            p { "{t.t(\"error-404-message\")}" }
        }
    }
}

#[component]
pub fn SignOutButton() -> Element {
    use auth::sign_out;

    let mut session = use_context::<Resource<Result<Option<User>>>>();
    let mut sign_out_action = use_action(sign_out);
    let nav = navigator();
    let t = i18n::use_translation();

    use_effect(move || {
        if sign_out_action.value().and_then(|r| r.ok()).is_some() {
            session.restart();
            nav.push(Route::Home {});
        }
    });

    rsx! {
        components::Button {
            variant: components::ButtonVariant::Secondary,
            size: components::ButtonSize::SM,
            loading: sign_out_action.pending(),
            onclick: move |_| sign_out_action.call(),
            if sign_out_action.pending() {
                "{t.t(\"auth-sign-out-loading\")}"
            } else {
                "{t.t(\"auth-sign-out-button\")}"
            }
        }
    }
}

/// Home page
#[component]
fn Home() -> Element {
    let realm = use_context::<Resource<Result<Realm>>>();
    let t = i18n::use_translation();

    rsx! {
        div { class: "flex min-h-full items-center justify-center p-8",
            h1 { class: "text-4xl font-bold text-gray-900 dark:text-gray-100",
                match realm() {
                    Some(Ok(Realm::Tenancy(tenant))) => rsx! { "{tenant.name}" },
                    Some(Ok(Realm::Platform)) => rsx! { "{t.t(\"home-welcome\")}" },
                    Some(Ok(Realm::UnknownTenant)) => rsx! { "{t.t(\"home-unknown-tenant\")}" },
                    Some(Err(_)) => rsx! { "{t.t(\"home-welcome\")}" },
                    None => rsx! { "{t.t(\"common-loading\")}" },
                }
            }
        }
    }
}

#[cfg(feature = "server")]
fn get_csrf_token_for_ssr() -> String {
    use dioxus::fullstack::FullstackContext;

    let ctx = match FullstackContext::current() {
        Some(ctx) => ctx,
        None => {
            tracing::warn!("SSR: FullstackContext not available");
            return String::new();
        }
    };

    // Get session to access CSRF token
    let session =
        match ctx.extension::<axum_session::Session<bits_axum_session_sqlx::SessionPgPool>>() {
            Some(s) => s,
            None => {
                tracing::warn!("SSR: Session not found in request extensions");
                return String::new();
            }
        };

    // Try to get existing token from session memory
    match session.get::<String>("csrf_token") {
        Some(token) => {
            tracing::debug!("SSR: Using existing CSRF token from session");
            token
        }
        None => {
            // Generate new token and store in session memory
            let token = csrf::generate_token();
            session.set("csrf_token", &token);
            tracing::debug!("SSR: Generated new CSRF token for session");
            token
        }
    }
}

/// Shared layout component with error handling.
#[component]
fn Layout() -> Element {
    use auth::{get_realm, get_session};

    let session = use_server_future(move || async move { get_session().await })?;
    let realm = use_server_future(move || async move { get_realm().await })?;
    let locale = i18n::create_default_locale()
        .unwrap_or_else(|e| panic!("Failed to create default locale: {}", e));

    use_context_provider(|| session);
    use_context_provider(|| realm);
    use_context_provider(|| locale);

    // TODO Make csrf_token a proper type akin to Option<String>.
    #[cfg(feature = "server")]
    let csrf_token = get_csrf_token_for_ssr();
    #[cfg(not(feature = "server"))]
    let csrf_token = String::new();

    rsx! {
        if !csrf_token.is_empty() {
            document::Meta { name: "csrf-token", content: "{csrf_token}" }
        }

        div { class: "flex min-h-screen flex-col",
            header { class: "bg-gray-100 dark:bg-gray-900",
                components::Header {}
            }
            main { class: "grow bg-white dark:bg-gray-900",
                ErrorBoundary {
                    handle_error: move |err: ErrorContext| {
                        let t = i18n::use_translation();

                        #[cfg(feature = "server")]
                        let http_error = dioxus::fullstack::FullstackContext::commit_error_status(
                            err.error().unwrap(),
                        );

                        #[cfg(not(feature = "server"))]
                        let http_error = err.error().unwrap();

                        rsx! {
                            div { class: "text-red-500",
                                h1 { "{t.t(\"common-error\")}" }
                                p { "{http_error:?}" }
                            }
                        }
                    },
                    Outlet::<Route> {}
                }
            }
            components::Footer {}
        }
    }
}
