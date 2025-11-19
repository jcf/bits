use dioxus::prelude::*;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Layout)]
    #[route("/")]
    Home {},
}

fn main() {
    dioxus::launch(App);
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
    rsx! {
        nav {
            class: "text-neutral-900 dark:text-neutral-100",
            a {
                href: "https://github.com/jcf/bits",
                class: "underline decoration-2 decoration-cyan-400",
                target: "_blank",
                "GitHub"
            }
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

/// Shared layout component.
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

        Outlet::<Route> {}
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
