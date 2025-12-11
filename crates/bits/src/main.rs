#![deny(clippy::fallible_impl_from)]
#![deny(clippy::fn_params_excessive_bools)]
#![deny(clippy::indexing_slicing)]
#![deny(clippy::must_use_candidate)]
#![deny(clippy::unneeded_field_pattern)]
#![deny(clippy::wildcard_enum_match_arm)]

#[cfg(feature = "server")]
use clap::{Parser, Subcommand};

#[cfg(feature = "server")]
mod server;

#[cfg(feature = "server")]
#[derive(Debug, Subcommand)]
enum Commands {
    Admin,
    Serve,
}

#[cfg(feature = "server")]
#[derive(Debug, Parser)]
#[command(name = "bits")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[command(flatten)]
    config: bits_app::Config,
}

#[cfg(feature = "server")]
fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Admin) => {
            println!("Hello, administrator.");
        }

        Some(Commands::Serve) | None => {
            bits_app::init_tracing();
            let config = cli.config.clone();
            dioxus::serve(move || {
                let config = config.clone();
                async move {
                    let state = bits_app::init(config).await?;
                    bits_app::build_router(state, server::app).await
                }
            });
        }
    }
}

#[cfg(not(feature = "server"))]
fn main() {
    #[cfg(target_arch = "wasm32")]
    bits_app::init_client();
    dioxus::launch(App);
}

#[cfg(not(feature = "server"))]
#[allow(non_snake_case)]
fn App() -> dioxus::prelude::Element {
    use dioxus::prelude::*;

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("assets/app.css") }
        bits_app::App {}
    }
}
