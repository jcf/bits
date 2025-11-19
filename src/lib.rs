#[cfg(feature = "server")]
pub mod db;
#[cfg(feature = "server")]
pub mod middleware;
#[cfg(feature = "server")]
pub mod tenant;

#[cfg(feature = "server")]
pub use db::{pool, TenantDb};
#[cfg(feature = "server")]
pub use middleware::tenant_middleware;
#[cfg(feature = "server")]
pub use tenant::{Tenant, TenantState};

// Re-export for client-side
#[cfg(not(feature = "server"))]
mod tenant_types {
    /// A validated tenant.
    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct Tenant {
        pub name: String,
    }
}

#[cfg(not(feature = "server"))]
pub use tenant_types::Tenant;

// App module
use dioxus::prelude::*;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Layout)]
    #[route("/")]
    Home {},
}

#[server]
async fn get_tenant() -> Result<Option<Tenant>, ServerFnError> {
    use dioxus::fullstack::{axum::Extension, FullstackContext, HttpError};

    let Extension(state): Extension<TenantState> = FullstackContext::extract().await?;

    match state {
        TenantState::Valid(tenant) => Ok(Some(tenant)),
        TenantState::Invalid(name) => HttpError::not_found(format!("Tenant not found: {}", name))?,
        TenantState::None => Ok(None),
    }
}

#[component]
pub fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: "data:" }
        Stylesheet { href: asset!("assets/app.css") }
        Router::<Route> {}
    }
}

#[component]
pub fn Hero() -> Element {
    let tenant = use_server_future(get_tenant)?.suspend()?().clone()?;

    rsx! {
        nav {
            class: "text-neutral-900 dark:text-neutral-100",
            match tenant {
                Some(t) => rsx! {
                    p { "Welcome, " strong { "{t.name}" } "!" }
                },
                None => rsx! {
                    p { "No tenant context" }
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
        Hero {}
        Echo {}
    }
}

/// Shared layout component with error handling.
#[component]
fn Layout() -> Element {
    rsx! {
        div {
            class: "text-neutral-900 dark:text-neutral-100",
            id: "navbar",
            Link {
                to: Route::Home {},
                "Home"
            }
        }

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

/// Echo component that demonstrates fullstack server functions.
#[component]
fn Echo() -> Element {
    let mut response = use_signal(|| String::new());

    rsx! {
        div {
            class: "text-neutral-900 dark:text-neutral-100",
            id: "echo",
            h4 { "ServerFn Echo" }
            input {
                placeholder: "Type here to echo...",
                oninput:  move |event| async move {
                    let data = echo_server(event.value()).await.unwrap();
                    response.set(data);
                },
            }

            if !response().is_empty() {
                p {
                    "Server echoed: "
                    i { "{response}" }
                }
            }
        }
    }
}

/// Echo the user input on the server.
#[server]
async fn echo_server(input: String) -> Result<String, ServerFnError> {
    Ok(input)
}
