#[cfg(feature = "server")]
use sqlx::PgPool;
use std::fmt;

/// A validated subdomain handle (e.g., "jcf" in "jcf.bits.page")
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Handle(String);

#[derive(Debug, thiserror::Error)]
pub enum HandleError {
    #[error("Handle must be 3-63 characters")]
    InvalidLength,
    #[error("Handle must contain only lowercase letters, numbers, and hyphens")]
    InvalidCharacters,
    #[error("Handle cannot start or end with a hyphen")]
    InvalidFormat,
}

impl Handle {
    /// Create a new handle with validation
    pub fn new(s: impl Into<String>) -> Result<Self, HandleError> {
        let s = s.into().to_lowercase();

        if s.len() < 3 || s.len() > 63 {
            return Err(HandleError::InvalidLength);
        }

        if !s
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(HandleError::InvalidCharacters);
        }

        if s.starts_with('-') || s.ends_with('-') {
            return Err(HandleError::InvalidFormat);
        }

        Ok(Handle(s))
    }

    /// Create from trusted source (e.g., database) without validation
    pub fn from_trusted(s: String) -> Self {
        Handle(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Handle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Handle {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// A validated tenant.
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Tenant {
    pub id: i64,
    pub name: String,
}

/// Realm represents the context for a request in multi-tenant mode
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum Realm {
    /// Platform realm - the apex domain
    Platform,
    /// Creator realm - a creator's site
    Creator(Tenant),
    /// Demo realm - a demo profile
    Demo(Handle),
    /// Not found - subdomain doesn't match any tenant or demo
    NotFound,
}

#[cfg(feature = "server")]
async fn load_tenant_by_domain(pool: &PgPool, domain: &str) -> Option<Tenant> {
    sqlx::query_as!(
        Tenant,
        "select
           t.id,
           t.name
         from tenants t
         join tenant_domains td on td.tenant_id = t.id
         where td.domain = $1
         and td.valid_to = 'infinity'",
        domain,
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

#[cfg(feature = "server")]
pub async fn resolve_realm(
    state: &crate::AppState,
    scheme: crate::http::Scheme,
    host: &str,
) -> Realm {
    let normalized_host = crate::http::normalize_host(scheme, host);

    #[cfg(feature = "colo")]
    {
        if let Some(ref platform_domain) = state.config.platform_domain {
            if normalized_host == *platform_domain {
                return Realm::Platform;
            }

            if normalized_host.ends_with(&format!(".{}", platform_domain)) {
                let subdomain = normalized_host
                    .strip_suffix(&format!(".{}", platform_domain))
                    .unwrap();

                let handle = Handle::from_trusted(subdomain.to_string());

                // Check if demo (no DB hit!)
                if crate::demos::SUBDOMAINS.contains(&handle.as_str()) {
                    return Realm::Demo(handle);
                }

                // Check database for real tenants
                if let Some(tenant) = load_tenant_by_domain(&state.db, &normalized_host).await {
                    return Realm::Creator(tenant);
                }
                return Realm::NotFound;
            }
        }
    }

    if let Some(tenant) = load_tenant_by_domain(&state.db, &normalized_host).await {
        return Realm::Creator(tenant);
    }

    Realm::NotFound
}

#[cfg(feature = "server")]
impl<S> axum_core::extract::FromRequestParts<S> for Realm
where
    S: Send + Sync,
{
    type Rejection = (dioxus::server::axum::http::StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut dioxus::server::axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts.extensions.get::<Realm>().cloned().ok_or((
            dioxus::server::axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Realm not set",
        ))
    }
}
