use bits_db::PostgresUrl;
use clap::Args;
use figment::{providers::Env, Figment};
use serde::{Deserialize, Serialize};

#[derive(Args, Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    #[arg(long, env = "DATABASE_URL")]
    pub database_url: PostgresUrl,

    #[arg(short, long, env = "MAX_DATABASE_CONNECTIONS", default_value = "5")]
    pub max_database_connections: u32,

    #[arg(short, long, env = "PORT", default_value = "3000")]
    pub port: u16,

    /// Platform domain (e.g., "bits.page") used for tenant routing
    #[cfg(feature = "colo")]
    #[arg(long, env = "PLATFORM_DOMAIN")]
    pub platform_domain: Option<String>,
}

impl Config {
    /// Load config from environment variables (for tests)
    pub fn from_env() -> Result<Self, Box<figment::Error>> {
        Figment::new()
            .merge(Env::prefixed(""))
            .extract()
            .map_err(Box::new)
    }

    pub fn with_database_url(mut self, url: PostgresUrl) -> Self {
        self.database_url = url;
        self
    }
}
