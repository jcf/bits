#[cfg(feature = "server")]
use sqlx::PgPool;

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
                if let Some(tenant) = load_tenant_by_domain(&state.db, &normalized_host).await {
                    return Realm::Tenancy(tenant);
                }
                return Realm::UnknownTenant;
            }
        }
    }

    if let Some(tenant) = load_tenant_by_domain(&state.db, &normalized_host).await {
        return Realm::Tenancy(tenant);
    }

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
        parts.extensions.get::<Realm>().cloned().ok_or((
            dioxus::server::axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Realm not set",
        ))
    }
}
