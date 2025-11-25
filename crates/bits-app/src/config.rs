use clap::Args;
use url::Url;

#[derive(Debug, Clone)]
pub struct PostgresUrl(Url);

impl PostgresUrl {
    pub fn parse(s: &str) -> Result<Self, PostgresUrlError> {
        let url = Url::parse(s)?;

        match url.scheme() {
            "postgres" | "postgresql" => {}
            _ => return Err(PostgresUrlError::NotPostgres),
        }

        if url.host_str().is_none() {
            return Err(PostgresUrlError::MissingHost);
        }

        Ok(Self(url))
    }
}

impl std::str::FromStr for PostgresUrl {
    type Err = PostgresUrlError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl AsRef<str> for PostgresUrl {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl std::ops::Deref for PostgresUrl {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PostgresUrlError {
    #[error("invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
    #[error("not a postgres:// or postgresql:// URL")]
    NotPostgres,
    #[error("missing host")]
    MissingHost,
}

#[derive(Args, Clone, Debug)]
pub struct Config {
    #[arg(long, env = "DATABASE_URL")]
    pub database_url: PostgresUrl,

    #[arg(short, long, env = "MAX_DATABASE_CONNECTIONS", default_value = "5")]
    pub max_database_connections: u32,

    #[arg(short, long, env = "PORT", default_value = "3000")]
    pub port: u16,

    /// Platform domain (e.g., "bits.page") for multi-tenant mode
    #[arg(long, env = "PLATFORM_DOMAIN")]
    pub platform_domain: Option<String>,

    /// Enable multi-tenant mode (domain-based tenant lookup)
    #[arg(long, env = "MULTI_TENANT", default_value = "false")]
    pub multi_tenant: bool,
}
