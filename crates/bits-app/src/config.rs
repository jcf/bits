#[cfg(not(target_arch = "wasm32"))]
use bits_db::PgUrl;
use clap::Args;
use figment::{providers::Env, Figment};
use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
fn default_max_database_connections() -> u32 {
    5
}

fn default_port() -> u16 {
    3000
}

#[derive(Args, Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// Dangerously allow JavaScript evaluation to allow hydration during
    /// development.
    #[arg(
        long,
        env = "DANGEROUSLY_ALLOW_JAVASCRIPT_EVALUATION",
        default_value = "false"
    )]
    #[serde(default)]
    pub dangerously_allow_javascript_evaluation: bool,

    #[cfg(not(target_arch = "wasm32"))]
    #[arg(long, env = "DATABASE_URL")]
    pub database_url: PgUrl,

    #[cfg(not(target_arch = "wasm32"))]
    #[arg(short, long, env = "MAX_DATABASE_CONNECTIONS", default_value = "5")]
    #[serde(default = "default_max_database_connections")]
    pub max_database_connections: u32,

    #[arg(short, long, env = "PORT", default_value = "3000")]
    #[serde(default = "default_port")]
    pub port: u16,

    /// Platform domain (e.g., "bits.page") used for tenant routing
    #[cfg(feature = "colo")]
    #[arg(long, env = "PLATFORM_DOMAIN")]
    pub platform_domain: Option<String>,
}

impl Config {
    /// Load config from environment variables
    pub fn from_env() -> Result<Self, Box<figment::Error>> {
        Figment::new()
            .merge(Env::prefixed(""))
            .extract()
            .map_err(Box::new)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn with_database_url(mut self, url: PgUrl) -> Self {
        self.database_url = url;
        self
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
}
