pub mod auth;
pub mod config;
pub mod http;
pub mod tenant;

#[cfg(feature = "server")]
pub mod middleware;
#[cfg(feature = "server")]
pub mod server;

pub use auth::{AuthError, AuthForm, JoinForm, User};
pub use config::Config;
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
pub fn Hero() -> Element {
    use auth::get_realm;

    let realm = use_server_future(move || async move { get_realm().await })?;
    let session = use_context::<Resource<Result<Option<User>>>>();

    rsx! {
        nav {
            class: "flex justify-between items-center text-neutral-900 dark:text-neutral-100 p-4",
            div {
                match realm() {
                    Some(Ok(Realm::Tenancy(tenant))) => rsx! {
                        p { "Tenant ID: {tenant.id}" }
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
            div {
                class: "flex gap-4 items-center",
                Link {
                    to: Route::Home {},
                    class: "underline decoration-2 decoration-cyan-400",
                    "Home"
                }
                match session() {
                    Some(Ok(Some(user))) => rsx! {
                        span { "{user.email}" }
                    },
                    _ => rsx! {
                        Link {
                            to: Route::Auth {},
                            class: "underline decoration-2 decoration-cyan-400",
                            "Sign in"
                        }
                        Link {
                            to: Route::Join {},
                            class: "underline decoration-2 decoration-cyan-400",
                            "Join"
                        }
                    },
                }
                a {
                    href: "https://github.com/jcf/bits",
                    class: "underline decoration-2 decoration-cyan-400",
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

    let mut session = use_context::<Resource<Result<Option<User>>>>();
    let mut auth_action = use_action(auth);
    let nav = navigator();

    rsx! {
        div {
            class: "flex min-h-full items-center justify-center px-4 py-12",
            div {
                class: "w-full max-w-sm space-y-10",
                h2 {
                    class: "mt-10 text-center text-2xl font-bold text-neutral-900 dark:text-white",
                    "Sign in to your account"
                }
                if let Some(Err(err)) = auth_action.value() {
                    div {
                        class: "text-red-500 text-sm text-center",
                        "{err}"
                    }
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
                        input {
                            id: "email",
                            r#type: "email",
                            name: "email",
                            required: true,
                            autocomplete: "off",
                            autocapitalize: "none",
                            placeholder: "Email address",
                            class: "block w-full rounded-md px-3 py-2 border border-neutral-300 dark:border-neutral-700"
                        }
                    }
                    div {
                        input {
                            id: "password",
                            r#type: "password",
                            name: "password",
                            required: true,
                            autocomplete: "off",
                            placeholder: "Password",
                            class: "block w-full rounded-md px-3 py-2 border border-neutral-300 dark:border-neutral-700"
                        }
                    }
                    button {
                        r#type: "submit",
                        disabled: auth_action.pending(),
                        class: "w-full rounded-md bg-indigo-600 px-3 py-2 text-white hover:bg-indigo-500",
                        if auth_action.pending() {
                            "Signing in..."
                        } else {
                            "Sign in"
                        }
                    }
                }
                p {
                    class: "text-center text-sm text-neutral-500",
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

    let mut join_action = use_action(join);
    let nav = navigator();

    rsx! {
        div {
            class: "flex min-h-full items-center justify-center px-4 py-12",
            div {
                class: "w-full max-w-sm space-y-10",
                h2 {
                    class: "mt-10 text-center text-2xl font-bold text-neutral-900 dark:text-white",
                    "Create your account"
                }
                if let Some(Err(err)) = join_action.value() {
                    div {
                        class: "text-red-500 text-sm text-center",
                        "{err}"
                    }
                }
                if let Some(Ok(_)) = join_action.value() {
                    div {
                        class: "text-green-500 text-sm text-center",
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
                        input {
                            id: "email",
                            r#type: "email",
                            name: "email",
                            required: true,
                            autocomplete: "off",
                            autocapitalize: "none",
                            placeholder: "Email address",
                            class: "block w-full rounded-md px-3 py-2 border border-neutral-300 dark:border-neutral-700"
                        }
                    }
                    div {
                        input {
                            id: "password",
                            r#type: "password",
                            name: "password",
                            required: true,
                            autocomplete: "off",
                            placeholder: "Password",
                            class: "block w-full rounded-md px-3 py-2 border border-neutral-300 dark:border-neutral-700"
                        }
                    }
                    button {
                        r#type: "submit",
                        disabled: join_action.pending(),
                        class: "w-full rounded-md bg-indigo-600 px-3 py-2 text-white hover:bg-indigo-500",
                        if join_action.pending() {
                            "Creating account..."
                        } else {
                            "Create account"
                        }
                    }
                }
                p {
                    class: "text-center text-sm text-neutral-500",
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
        div {
            class: "text-red-500",
            h1 { "404 - Tenant Not Found" }
            p { "The requested tenant does not exist." }
        }
    }
}

/// Home page
#[component]
fn Home() -> Element {
    rsx! {
        div {
            class: "flex min-h-full items-center justify-center p-8",
            h1 {
                class: "text-4xl font-bold",
                "Welcome to Bits"
            }
        }
    }
}

/// Shared layout component with error handling.
#[component]
fn Layout() -> Element {
    use auth::get_session;

    let session = use_server_future(move || async move { get_session().await })?;
    use_context_provider(|| session);

    rsx! {
        div {
            class: "flex min-h-screen flex-col",
            div {
                class: "sticky top-0 bg-neutral-100 dark:bg-neutral-900",
                Hero {}
            }
            div {
                class: "flex-grow",
                ErrorBoundary {
                    handle_error: move |err: ErrorContext| {
                        #[cfg(feature = "server")]
                        let http_error = dioxus::fullstack::FullstackContext::commit_error_status(
                            err.error().unwrap()
                        );

                        #[cfg(not(feature = "server"))]
                        let http_error = err.error().unwrap();

                        rsx! {
                            div {
                                class: "text-red-500",
                                h1 { "Error" }
                                p { "{http_error:?}" }
                            }
                        }
                    },
                    Outlet::<Route> {}
                }
            }
        }
    }
}
