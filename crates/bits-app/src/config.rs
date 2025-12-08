#[cfg(not(target_arch = "wasm32"))]
use bits_db::PgUrl;
use clap::Args;
use figment::{providers::Env, Figment};
#[cfg(not(target_arch = "wasm32"))]
use garde::Validate;
use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to parse configuration: {0}")]
    Parse(#[from] Box<figment::Error>),
    #[cfg(not(target_arch = "wasm32"))]
    #[error("Configuration validation failed: {0}")]
    Validation(#[from] garde::Report),
}

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

// TODO Wrap u32 in 'Limit' type.
#[cfg(not(target_arch = "wasm32"))]
fn default_global_rate_limit() -> Option<u32> {
    Some(50)
}

#[derive(Args, Clone, Debug, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Validate))]
pub struct Config {
    /// Application version (typically git commit hash)
    #[arg(long, env = "BITS_VERSION", default_value = "dev")]
    #[serde(default = "default_version")]
    #[cfg_attr(not(target_arch = "wasm32"), garde(skip))]
    pub version: String,
    /// Dangerously allow JavaScript evaluation to allow hydration during
    /// development.
    #[arg(
        long,
        env = "DANGEROUSLY_ALLOW_JAVASCRIPT_EVALUATION",
        default_value = "false"
    )]
    #[serde(default)]
    #[cfg_attr(not(target_arch = "wasm32"), garde(skip))]
    pub dangerously_allow_javascript_evaluation: bool,

    #[cfg(not(target_arch = "wasm32"))]
    #[arg(long, env = "DATABASE_URL")]
    #[garde(skip)]
    pub database_url: PgUrl,

    #[cfg(not(target_arch = "wasm32"))]
    #[arg(short, long, env = "MAX_DATABASE_CONNECTIONS", default_value = "5")]
    #[serde(default = "default_max_database_connections")]
    #[garde(skip)]
    pub max_database_connections: u32,

    /// Argon2 memory cost in KiB (default: 122880 = 120 MiB)
    #[cfg(not(target_arch = "wasm32"))]
    #[arg(long, env = "ARGON2_M_COST", default_value = "122880")]
    #[serde(default = "default_argon2_m_cost")]
    #[garde(skip)]
    pub argon2_m_cost: u32,

    /// Argon2 time cost (iterations, default: 3)
    #[cfg(not(target_arch = "wasm32"))]
    #[arg(long, env = "ARGON2_T_COST", default_value = "3")]
    #[serde(default = "default_argon2_t_cost")]
    #[garde(skip)]
    pub argon2_t_cost: u32,

    /// Argon2 parallelism (threads, default: 4)
    #[cfg(not(target_arch = "wasm32"))]
    #[arg(long, env = "ARGON2_P_COST", default_value = "4")]
    #[serde(default = "default_argon2_p_cost")]
    #[garde(skip)]
    pub argon2_p_cost: u32,

    /// Master key for deriving application-specific secrets (required)
    #[cfg(not(target_arch = "wasm32"))]
    #[arg(long, env = "MASTER_KEY")]
    #[garde(skip)]
    pub master_key: String,

    #[arg(short, long, env = "PORT", default_value = "3000")]
    #[serde(default = "default_port")]
    #[cfg_attr(not(target_arch = "wasm32"), garde(skip))]
    pub port: u16,

    /// Session cookie name
    #[arg(long, env = "SESSION_NAME", default_value = "bits")]
    #[serde(default = "default_session_name")]
    #[cfg_attr(not(target_arch = "wasm32"), garde(skip))]
    pub session_name: String,

    /// Global rate limit (requests per second per IP, None = disabled)
    #[cfg(not(target_arch = "wasm32"))]
    #[arg(long, env = "GLOBAL_RATE_LIMIT")]
    #[serde(default = "default_global_rate_limit")]
    #[garde(inner(range(min = 1, max = 1000)))]
    pub global_rate_limit: Option<u32>,

    /// Bearer token for /metrics endpoint (optional - if not set, endpoint is unprotected)
    #[cfg(not(target_arch = "wasm32"))]
    #[arg(long, env = "METRICS_AUTH_TOKEN")]
    #[serde(default)]
    #[garde(skip)]
    pub metrics_auth_token: Option<String>,

    /// Platform domain (e.g., "bits.page") used for tenant routing
    #[arg(long, env = "PLATFORM_DOMAIN")]
    #[cfg_attr(not(target_arch = "wasm32"), garde(skip))]
    pub platform_domain: String,
}

impl Config {
    /// Load config from environment variables
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_env() -> Result<Self, ConfigError> {
        let config: Self = Figment::new()
            .merge(Env::prefixed(""))
            .extract()
            .map_err(|e| ConfigError::Parse(Box::new(e)))?;

        config.validate()?;

        Ok(config)
    }

    /// Load config from environment variables (WASM version without validation)
    #[cfg(target_arch = "wasm32")]
    pub fn from_env() -> Result<Self, ConfigError> {
        Figment::new()
            .merge(Env::prefixed(""))
            .extract()
            .map_err(|e| ConfigError::Parse(Box::new(e)))
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[must_use]
    pub fn with_database_url(mut self, url: PgUrl) -> Self {
        self.database_url = url;
        self
    }

    #[must_use]
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[must_use]
    pub fn with_test_argon2_params(mut self) -> Self {
        self.argon2_m_cost = 8; // 8 KiB (minimal, fast for tests)
        self.argon2_t_cost = 1; // 1 iteration
        self.argon2_p_cost = 1; // 1 thread
        self
    }
}

/// Load config from environment variables
#[cfg(not(target_arch = "wasm32"))]
pub fn load_config() -> Result<Config, ConfigError> {
    Config::from_env()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_defaults_to_dev() {
        let config = Config::from_env().expect("Unable to configure from env");
        assert_eq!(config.version, "dev");
    }

    #[cfg(not(target_arch = "wasm32"))]
    mod validation {
        use super::*;
        use garde::Validate;
        use rstest::rstest;

        fn config() -> Config {
            Config {
                argon2_m_cost: 19456,
                argon2_p_cost: 1,
                argon2_t_cost: 2,
                dangerously_allow_javascript_evaluation: false,
                database_url: "postgresql://localhost/test".parse().unwrap(),
                global_rate_limit: Some(50),
                master_key: "test-master-key-32-bytes-long!!".to_string(),
                max_database_connections: 5,
                metrics_auth_token: None,
                platform_domain: "test.local".to_string(),
                port: 3000,
                session_name: "test".to_string(),
                version: "test".to_string(),
            }
        }

        #[rstest]
        #[case(None)]
        #[case(Some(1))]
        #[case(Some(50))]
        #[case(Some(1000))]
        fn global_rate_limit_valid(#[case] limit: Option<u32>) {
            let result = Config {
                global_rate_limit: limit,
                ..config()
            }
            .validate();
            assert!(result.is_ok());
        }

        #[rstest]
        #[case(Some(0))]
        #[case(Some(1001))]
        #[case(Some(9999))]
        fn global_rate_limit_invalid(#[case] limit: Option<u32>) {
            let result = Config {
                global_rate_limit: limit,
                ..config()
            }
            .validate();

            assert!(result.is_err());
            let report = result.unwrap_err();
            assert!(report
                .iter()
                .any(|(path, _)| path.to_string().contains("global_rate_limit")));
        }
    }
}
