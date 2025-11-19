use dioxus::prelude::*;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Layout)]
    #[route("/")]
    Home {},
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Tenant {
    pub name: String,
}

/// Result of tenant resolution from the request
#[derive(Clone, Debug)]
pub enum TenantState {
    /// Valid tenant found
    Valid(Tenant),
    /// Tenant subdomain present but not recognized
    Invalid(String),
    /// No tenant context (e.g., accessing bits.page.test directly)
    None,
}

fn main() {
    #[cfg(feature = "server")]
    {
        use dioxus::server::axum::{self, extract::Request, middleware::Next, response::Response};

        async fn tenant_middleware(req: Request, next: Next) -> Response {
            let host = req
                .headers()
                .get("host")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("")
                .to_string();

            // Extract tenant from subdomain of bits.page.test
            let tenant = if host.ends_with(".bits.page.test") {
                host.strip_suffix(".bits.page.test").map(|s| s.to_string())
            } else {
                None
            };

            let valid_tenants = ["jcf"];

            let tenant_state = match tenant {
                Some(name) if valid_tenants.contains(&name.as_str()) => {
                    TenantState::Valid(Tenant { name })
                }
                Some(name) => TenantState::Invalid(name),
                None => TenantState::None,
            };

            let mut req = req;
            req.extensions_mut().insert(tenant_state);

            next.run(req).await
        }

        dioxus::serve(|| async move {
            // Future: setup database pool, run migrations, etc.

            Ok(dioxus::server::router(App).layer(axum::middleware::from_fn(tenant_middleware)))
        });
    }

    #[cfg(not(feature = "server"))]
    dioxus::launch(App);
}

#[server]
async fn get_tenant() -> Result<Option<Tenant>, ServerFnError> {
    use dioxus::fullstack::{axum::Extension, extract, HttpError};

    let Extension(state): Extension<TenantState> = extract().await?;

    match state {
        TenantState::Valid(tenant) => Ok(Some(tenant)),
        TenantState::Invalid(name) => HttpError::not_found(format!("Tenant not found: {}", name))?,
        TenantState::None => Ok(None),
    }
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: "data:" }
        Stylesheet { href: asset!("assets/app.css") }
        Router::<Route> {}
    }
}

#[component]
pub fn Hero() -> Element {
    // suspend() handles loading state, ? propagates errors to ErrorBoundary
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
