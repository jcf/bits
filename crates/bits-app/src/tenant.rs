#[cfg(feature = "server")]
use crate::Config;
#[cfg(feature = "server")]
use sqlx::PgPool;

/// A validated tenant.
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Tenant {
    pub id: i64,
}

/// Realm represents the context for a request in multi-tenant mode
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum Realm {
    /// Platform realm - the apex domain
    Platform,
    /// Tenancy realm - a creator's site
    Tenancy(Tenant),
    /// Unknown subdomain - subdomain doesn't match any tenant
    UnknownTenant,
}

#[cfg(feature = "server")]
async fn load_tenant_by_domain(pool: &PgPool, domain: &str) -> Option<Tenant> {
    sqlx::query_as::<_, Tenant>(
        "select t.id
         from tenants t
         join tenant_domains td on td.tenant_id = t.id
         where td.domain = $1
           and td.valid_to = 'infinity'",
    )
    .bind(domain)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

#[cfg(feature = "server")]
pub async fn resolve_realm(host: &str, config: &Config, db: &PgPool) -> Realm {
    // Check if this is the platform domain
    if let Some(ref platform_domain) = config.platform_domain {
        // Strip port if present for comparison
        let host_without_port = host.split(':').next().unwrap_or(host);

        if host_without_port == platform_domain {
            return Realm::Platform;
        }

        // Check if it's a subdomain of the platform
        if host_without_port.ends_with(&format!(".{}", platform_domain)) {
            // Try to find a tenant by domain (without port)
            if let Some(tenant) = load_tenant_by_domain(db, host_without_port).await {
                return Realm::Tenancy(tenant);
            }
            // Subdomain pattern but no tenant found
            return Realm::UnknownTenant;
        }
    }

    // Try to find a tenant by custom domain (without port)
    let host_without_port = host.split(':').next().unwrap_or(host);
    if let Some(tenant) = load_tenant_by_domain(db, host_without_port).await {
        return Realm::Tenancy(tenant);
    }

    // Unknown domain, treat as unknown tenant
    Realm::UnknownTenant
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
        parts
            .extensions
            .get::<Realm>()
            .cloned()
            .ok_or((
                dioxus::server::axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Realm not set",
            ))
    }
}
