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
            dioxus::serve(|| server::server(cli.config.clone()));
        }
    }
}

#[cfg(not(feature = "server"))]
fn main() {
    dioxus::launch(App);
}

#[cfg(not(feature = "server"))]
fn App() -> dioxus::prelude::Element {
    use dioxus::prelude::*;

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("assets/app.css") }
        bits_app::App {}
    }
}
