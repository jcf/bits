use clap::Args;

#[derive(Args, Clone, Debug)]
pub struct Config {
    #[arg(short, long, env = "PORT", default_value = "8080")]
    pub port: u16,

    #[arg(long, env = "DATABASE_URL")]
    pub database_url: String,
}
