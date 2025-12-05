#[cfg(not(target_arch = "wasm32"))]
use bits_db::PgUrl;
use clap::Args;
use figment::{providers::Env, Figment};
use serde::Deserialize;

#[cfg(not(target_arch = "wasm32"))]
fn default_max_database_connections() -> u32 {
    5
}

fn default_port() -> u16 {
    3000
}

fn default_version() -> String {
    option_env!("BITS_VERSION").unwrap_or("dev").to_string()
}

fn default_session_name() -> String {
    "bits".to_string()
}

#[cfg(not(target_arch = "wasm32"))]
fn default_argon2_m_cost() -> u32 {
    122880 // 120 MiB - Server-grade memory cost per OWASP recommendations
}

#[cfg(not(target_arch = "wasm32"))]
fn default_argon2_t_cost() -> u32 {
    3 // 3 iterations - Balance between security and performance
}

#[cfg(not(target_arch = "wasm32"))]
fn default_argon2_p_cost() -> u32 {
    4 // 4 threads - Leverage multi-core processors
}

#[derive(Args, Clone, Debug, Deserialize)]
pub struct Config {
    /// Application version (typically git commit hash)
    #[arg(long, env = "BITS_VERSION", default_value = "dev")]
    #[serde(default = "default_version")]
    pub version: String,
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

    /// Argon2 memory cost in KiB (default: 122880 = 120 MiB)
    #[cfg(not(target_arch = "wasm32"))]
    #[arg(long, env = "ARGON2_M_COST", default_value = "122880")]
    #[serde(default = "default_argon2_m_cost")]
    pub argon2_m_cost: u32,

    /// Argon2 time cost (iterations, default: 3)
    #[cfg(not(target_arch = "wasm32"))]
    #[arg(long, env = "ARGON2_T_COST", default_value = "3")]
    #[serde(default = "default_argon2_t_cost")]
    pub argon2_t_cost: u32,

    /// Argon2 parallelism (threads, default: 4)
    #[cfg(not(target_arch = "wasm32"))]
    #[arg(long, env = "ARGON2_P_COST", default_value = "4")]
    #[serde(default = "default_argon2_p_cost")]
    pub argon2_p_cost: u32,

    /// Master key for encryption and HMAC operations (base64-encoded 64-byte key)
    /// REQUIRED: Must be set via MASTER_KEY environment variable
    #[cfg(not(target_arch = "wasm32"))]
    #[arg(long, env = "MASTER_KEY")]
    pub master_key: String,

    #[arg(short, long, env = "PORT", default_value = "3000")]
    #[serde(default = "default_port")]
    pub port: u16,

    /// Session cookie name
    #[arg(long, env = "SESSION_NAME", default_value = "bits")]
    #[serde(default = "default_session_name")]
    pub session_name: String,

    /// Bearer token for /metrics endpoint (optional - if not set, endpoint is unprotected)
    #[cfg(not(target_arch = "wasm32"))]
    #[arg(long, env = "METRICS_AUTH_TOKEN")]
    #[serde(default)]
    pub metrics_auth_token: Option<String>,

    /// Platform domain (e.g., "bits.page") used for tenant routing
    #[arg(long, env = "PLATFORM_DOMAIN")]
    #[serde(default)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_defaults_to_dev() {
        let config = Config::from_env().expect("Unable to configure from env");
        assert_eq!(config.version, "dev");
    }
}
