use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PgUrl(Url);

impl PgUrl {
    pub fn parse(s: &str) -> Result<Self, PgUrlError> {
        let url = Url::parse(s)?;

        match url.scheme() {
            "postgres" | "postgresql" => {}
            _ => return Err(PgUrlError::NotPostgres),
        }

        if url.host_str().is_none() {
            return Err(PgUrlError::MissingHost);
        }

        Ok(Self(url))
    }

    #[must_use]
    pub fn database(&self) -> Option<&str> {
        self.0.path().strip_prefix('/')
    }

    #[must_use]
    pub fn with_database(&self, db: &str) -> Self {
        let mut url = self.0.clone();
        url.set_path(&format!("/{}", db));
        Self(url)
    }
}

impl std::str::FromStr for PgUrl {
    type Err = PgUrlError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl AsRef<str> for PgUrl {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl std::ops::Deref for PgUrl {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PgUrlError {
    #[error("invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
    #[error("not a postgres:// or postgresql:// URL")]
    NotPostgres,
    #[error("missing host")]
    MissingHost,
}
