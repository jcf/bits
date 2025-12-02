use bits_app::Config;
use dioxus::server::axum;

#[allow(non_snake_case)]
fn App() -> dioxus::prelude::Element {
    use dioxus::prelude::*;

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("assets/app.css") }
        bits_app::App {}
    }
}

pub async fn router(config: Config) -> Result<axum::Router, anyhow::Error> {
    bits_app::router(config, App).await
}
