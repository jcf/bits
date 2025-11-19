use axum_core::extract::{FromRef, FromRequest};
use bits::components::Container;
use dioxus::{fullstack::FullstackContext, prelude::*};
use reqwest::header::HeaderMap;

#[cfg(feature = "server")]
use {
    dioxus::fullstack::axum,
    dioxus::fullstack::Lazy,
    futures::lock::Mutex,
    sqlx::{Executor, Row},
    std::sync::LazyLock,
};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Layout)]
    #[route("/")]
    Home {},
}

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[derive(Clone)]
struct AppState {
    abc: i32,
}

impl FromRef<FullstackContext> for AppState {
    fn from_ref(state: &FullstackContext) -> Self {
        state.extension::<AppState>().unwrap()
    }
}

struct CustomExtractor {
    abc: i32,
    headermap: HeaderMap,
}

impl<S> FromRequest<S> for CustomExtractor
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ();

    async fn from_request(
        _req: axum::extract::Request,
        state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);
        Ok(CustomExtractor {
            abc: state.abc,
            headermap: HeaderMap::new(),
        })
    }
}

fn main() {
    #[cfg(not(feature = "server"))]
    dioxus::launch(app);

    #[cfg(feature = "server")]
    dioxus::serve(|| async move {
        use dioxus::server::axum::Extension;

        let router = dioxus::server::router(app)
            .layer(Extension(tokio::sync::broadcast::channel::<String>(16).0));
        let router = router.layer(Extension(AppState { abc: 42 }));

        Ok(router)
    });
}

#[component]
fn app() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Router::<Route> {}
    }
}

#[component]
fn Header() -> Element {
    rsx! {
        header {
            class: "text-neutral-900 dark:text-neutral-100",
            Container {
                Link { to: Route::Home {}, "Bits" }
            }
        }
    }
}

#[component]
fn Footer() -> Element {
    rsx! {
        footer {
            class: "text-neutral-900 dark:text-neutral-100",
            Container {
                nav {
                  a { href: "https://github.com/jcf/bits", target: "_blank", "GitHub" }
                }
            }
        }
    }
}

#[component]
fn Layout() -> Element {
    rsx! {
        Header {}

        main {
            class: "flex-1",
            Outlet::<Route> {}
        }

        Footer {}
    }
}

#[get("/api/hello")]
async fn hello_world() -> Result<String, ServerFnError> {
    Ok("Hello world!".to_string())
}

#[component]
fn Home() -> Element {
    let onclick = move |_| async move {
        if let Ok(msg) = hello_world().await {
            println!("hello_world says {}", msg);
        }
    };

    rsx! {
        Container {
            h1 { class: "text-neutral-900 dark:text-neutral-100", "Coming soon." }
            a { onclick: onclick, "Salute" }
        }
    }
}
