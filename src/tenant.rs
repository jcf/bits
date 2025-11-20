use sqlx::PgPool;

/// A validated tenant.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Tenant {
    pub name: String,
}

/// Result of tenant resolution from the request.
#[derive(Clone, Debug)]
pub enum TenantState {
    /// Valid tenant found
    Valid(Tenant),
    /// Tenant subdomain present but not recognized
    Invalid(String),
    /// No tenant context (e.g., accessing bits.page.test directly)
    None,
}

/// Extract tenant subdomain from host header.
pub fn extract_tenant_from_host(host: &str) -> Option<String> {
    if host.ends_with(".bits.page.test") {
        host.strip_suffix(".bits.page.test").map(|s| s.to_string())
    } else {
        None
    }
}

/// Check if a tenant exists in the database.
pub async fn tenant_exists(pool: &PgPool, name: &str) -> Result<bool, sqlx::Error> {
    let exists: Option<bool> =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM public.tenants WHERE name = $1)")
            .bind(name)
            .fetch_one(pool)
            .await?;

    Ok(exists.unwrap_or(false))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tenant_from_host() {
        assert_eq!(
            extract_tenant_from_host("jcf.bits.page.test"),
            Some("jcf".to_string())
        );
        assert_eq!(
            extract_tenant_from_host("acme.bits.page.test"),
            Some("acme".to_string())
        );
        assert_eq!(extract_tenant_from_host("bits.page.test"), None);
        assert_eq!(extract_tenant_from_host("example.com"), None);
        assert_eq!(extract_tenant_from_host(""), None);
    }

    #[tokio::test]
    async fn test_tenant_exists() {
        use crate::config::Config;
        use crate::db::pool;

        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
        let config = Config {
            port: 8080,
            database_url,
        };
        let _ = crate::init(config);

        let pool = pool().await;

        // jcf should exist (seeded in migration)
        assert!(tenant_exists(pool, "jcf").await.unwrap());

        // nonexistent tenant
        assert!(!tenant_exists(pool, "nonexistent").await.unwrap());
    }
}
