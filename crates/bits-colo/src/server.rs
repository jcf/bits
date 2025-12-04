use dioxus::prelude::*;

#[allow(non_snake_case)]
pub fn app() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("assets/app.css") }
        bits_app::App {}
    }
}
