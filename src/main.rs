use bits::App;
use clap::{Parser, Subcommand};

#[cfg(feature = "server")]
use bits::{config::Config, tenant_middleware};

#[derive(Parser)]
#[command(name = "bits")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[cfg(feature = "server")]
    #[command(flatten)]
    config: Config,
}

#[derive(Subcommand)]
enum Commands {
    Admin,
    Serve,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Admin) => {
            println!("Hello, administrator.");
            return;
        }
        Some(Commands::Serve) | None => {
            #[cfg(feature = "server")]
            {
                std::env::set_var("PORT", cli.config.port.to_string());
                bits::init(cli.config);
            }
        }
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
