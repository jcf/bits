use std::sync::Arc;

use dioxus::server::axum::{extract::Request, middleware::Next, response::Response};
use tokio::sync::Mutex;

use crate::db::{pool, TenantDb};
use crate::tenant::{extract_tenant_from_host, tenant_exists, Tenant, TenantState};

/// Middleware that resolves tenant from host and sets up database connection.
pub async fn tenant_middleware(mut req: Request, next: Next) -> Response {
    let host = req
        .headers()
        .get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("")
        .to_string();

    let tenant = extract_tenant_from_host(&host);
    let pool = pool().await;

    let tenant_state = match tenant {
        Some(name) => {
            let exists = tenant_exists(pool, &name).await.unwrap_or(false);

            if exists {
                TenantState::Valid(Tenant { name })
            } else {
                TenantState::Invalid(name)
            }
        }
        None => TenantState::None,
    };

    // Set up database connection with tenant schema
    if let TenantState::Valid(ref tenant) = tenant_state {
        if let Ok(mut conn) = pool.acquire().await {
            let schema_query = format!("SET search_path TO tenant_{}, public", tenant.name);
            if sqlx::query(&schema_query).execute(&mut *conn).await.is_ok() {
                req.extensions_mut()
                    .insert(TenantDb(Arc::new(Mutex::new(conn))));
            }
        }
    }

    req.extensions_mut().insert(tenant_state);
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_state_variants() {
        let valid = TenantState::Valid(Tenant {
            name: "test".to_string(),
        });
        let invalid = TenantState::Invalid("unknown".to_string());
        let none = TenantState::None;

        // Just ensure we can create all variants
        match valid {
            TenantState::Valid(t) => assert_eq!(t.name, "test"),
            _ => panic!("Expected Valid"),
        }

        match invalid {
            TenantState::Invalid(name) => assert_eq!(name, "unknown"),
            _ => panic!("Expected Invalid"),
        }

        assert!(matches!(none, TenantState::None));
    }
}
