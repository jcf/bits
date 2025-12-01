pub mod auth;
pub mod components;
pub mod config;
pub mod http;
pub mod tenant;

#[cfg(feature = "server")]
pub mod middleware;
#[cfg(feature = "server")]
pub mod server;

pub use auth::{AuthError, AuthForm, JoinForm, User};
pub use config::Config;
pub use http::CspMode;
pub use tenant::{Realm, Tenant};

#[cfg(feature = "server")]
pub use middleware::{RealmLayer, RealmMiddleware};
#[cfg(feature = "server")]
pub use server::{init, init_tracing, setup_session_store};

#[cfg(feature = "server")]
use dioxus::fullstack::FullstackContext;

#[cfg(feature = "server")]
#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Config,
    pub db: sqlx::PgPool,
}

#[cfg(feature = "server")]
impl AppState {
    pub async fn new(config: Config) -> Result<Self, anyhow::Error> {
        let db = sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.max_database_connections)
            .connect(config.database_url.as_ref())
            .await?;

        Ok(Self { config, db })
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
pub fn Header() -> Element {
    let realm = use_context::<Resource<Result<Realm>>>();
    let session = use_context::<Resource<Result<Option<User>>>>();

    rsx! {
        nav { class: "flex justify-between items-center text-neutral-900 dark:text-neutral-100 p-4",
            div {
                match realm() {
                    Some(Ok(Realm::Tenancy(tenant))) => rsx! {
                        document::Title { "{tenant.name}" }
                        h1 { "{tenant.name}" }
                    },
                    Some(Ok(Realm::Platform)) => rsx! {
                        p { "Platform" }
                    },
                    Some(Ok(Realm::UnknownTenant)) => rsx! {
                        p { class: "text-red-500", "Unknown subdomain" }
                    },
                    Some(Err(_)) => rsx! {
                        p { class: "text-red-500", "Error loading realm" }
                    },
                    None => rsx! {
                        p { "Loading…" }
                    },
                }
            }
            div { class: "flex gap-4 items-center",
                Link {
                    to: Route::Home {},
                    class: "underline decoration-2 decoration-indigo-400",
                    "Home"
                }
                match session() {
                    Some(Ok(Some(user))) => rsx! {
                        span { "{user.email}" }
                        SignOutButton {}
                    },
                    _ => rsx! {
                        components::ButtonLink {
                            to: Route::Auth {},
                            variant: components::ButtonVariant::Secondary,
                            size: components::ButtonSize::SM,
                            "Sign in"
                        }
                        components::ButtonLink {
                            to: Route::Join {},
                            variant: components::ButtonVariant::Primary,
                            size: components::ButtonSize::SM,
                            "Join"
                        }
                    },
                }
                a {
                    href: "https://github.com/jcf/bits",
                    class: "underline decoration-2 decoration-indigo-400",
                    target: "_blank",
                    "GitHub"
                }
            }
        }
    }
}

#[component]
fn Auth() -> Element {
    use auth::{auth, AuthForm};
    use components::Input;

    let mut session = use_context::<Resource<Result<Option<User>>>>();
    let mut auth_action = use_action(auth);
    let nav = navigator();

    rsx! {
        div { class: "flex min-h-full items-center justify-center px-4 py-12",
            div { class: "w-full max-w-sm space-y-10",
                h2 { class: "mt-10 text-center text-2xl font-bold text-neutral-900 dark:text-white",
                    "Sign in to your account"
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
                            placeholder: "Email address",
                        }
                    }
                    div {
                        Input {
                            id: "password",
                            input_type: "password",
                            name: "password",
                            placeholder: "Password",
                        }
                    }
                    components::Button {
                        button_type: "submit",
                        variant: components::ButtonVariant::Primary,
                        size: components::ButtonSize::LG,
                        loading: auth_action.pending(),
                        class: "w-full",
                        if auth_action.pending() {
                            "Signing in..."
                        } else {
                            "Sign in"
                        }
                    }
                }
                p { class: "text-center text-sm text-neutral-500",
                    "Not a member? "
                    Link {
                        to: Route::Join {},
                        class: "text-indigo-600 hover:text-indigo-500",
                        "Create an account"
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

    rsx! {
        div { class: "flex min-h-full items-center justify-center px-4 py-12",
            div { class: "w-full max-w-sm space-y-10",
                h2 { class: "mt-10 text-center text-2xl font-bold text-neutral-900 dark:text-white",
                    "Create your account"
                }
                if let Some(Err(err)) = join_action.value() {
                    div { class: "text-red-500 text-sm text-center", "{err}" }
                }
                if let Some(Ok(_)) = join_action.value() {
                    div { class: "text-green-500 text-sm text-center",
                        "Account created! You can now sign in once your email is verified."
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
                            placeholder: "Email address",
                        }
                    }
                    div {
                        Input {
                            id: "password",
                            input_type: "password",
                            name: "password",
                            placeholder: "Password",
                        }
                    }
                    components::Button {
                        button_type: "submit",
                        variant: components::ButtonVariant::Primary,
                        size: components::ButtonSize::LG,
                        loading: join_action.pending(),
                        class: "w-full",
                        if join_action.pending() {
                            "Creating account..."
                        } else {
                            "Create account"
                        }
                    }
                }
                p { class: "text-center text-sm text-neutral-500",
                    "Already a member? "
                    Link {
                        to: Route::Auth {},
                        class: "text-indigo-600 hover:text-indigo-500",
                        "Sign in"
                    }
                }
            }
        }
    }
}

#[component]
fn NotFound() -> Element {
    rsx! {
        div { class: "text-red-500",
            h1 { "404 - Tenant Not Found" }
            p { "The requested tenant does not exist." }
        }
    }
}

#[component]
pub fn SignOutButton() -> Element {
    use auth::sign_out;

    let mut session = use_context::<Resource<Result<Option<User>>>>();
    let mut sign_out_action = use_action(sign_out);
    let nav = navigator();

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
                "Signing out..."
            } else {
                "Sign out"
            }
        }
    }
}

/// Home page
#[component]
fn Home() -> Element {
    let realm = use_context::<Resource<Result<Realm>>>();

    rsx! {
        div { class: "flex min-h-full items-center justify-center p-8",
            h1 { class: "text-4xl font-bold text-neutral-900 dark:text-neutral-100",
                match realm() {
                    Some(Ok(Realm::Tenancy(tenant))) => rsx! { "{tenant.name}" },
                    Some(Ok(Realm::Platform)) => rsx! { "Welcome to Bits" },
                    Some(Ok(Realm::UnknownTenant)) => rsx! { "Unknown Tenant" },
                    Some(Err(_)) => rsx! { "Welcome to Bits" },
                    None => rsx! { "Loading…" },
                }
            }
        }
    }
}

/// Shared layout component with error handling.
#[component]
fn Layout() -> Element {
    use auth::{get_realm, get_session};

    let session = use_server_future(move || async move { get_session().await })?;
    let realm = use_server_future(move || async move { get_realm().await })?;
    use_context_provider(|| session);
    use_context_provider(|| realm);

    rsx! {
        div { class: "flex min-h-screen flex-col",
            header { class: "bg-neutral-100 dark:bg-neutral-900", Header {} }
            main { class: "grow",
                ErrorBoundary {
                    handle_error: move |err: ErrorContext| {
                        #[cfg(feature = "server")]
                        let http_error = dioxus::fullstack::FullstackContext::commit_error_status(
                            err.error().unwrap(),
                        );

                        #[cfg(not(feature = "server"))]
                        let http_error = err.error().unwrap();

                        rsx! {
                            div { class: "text-red-500",
                                h1 { "Error" }
                                p { "{http_error:?}" }
                            }
                        }
                    },
                    Outlet::<Route> {}
                }
            }
            footer {
                p { "Powered by Bits" }
            }
        }
    }
}
