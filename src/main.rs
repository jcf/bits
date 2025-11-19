use bits::App;
use clap::{Parser, Subcommand};

#[cfg(feature = "server")]
use bits::tenant_middleware;

#[derive(Parser)]
#[command(name = "bits")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Admin,
    Serve {
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Admin) => {
            println!("Hello, administrator.");
            return;
        }
        Some(Commands::Serve { port }) => {
            std::env::set_var("PORT", port.to_string());
        }
        None => {}
    }

    #[cfg(feature = "server")]
    {
        use dioxus::server::axum;

        dioxus::serve(|| async move {
            Ok(dioxus::server::router(App).layer(axum::middleware::from_fn(tenant_middleware)))
        });
    }

    #[cfg(not(feature = "server"))]
    dioxus::launch(App);
}
