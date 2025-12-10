use bits_app::{http::Scheme, tenant::Realm};
use bits_e2e::fixtures;

#[tokio::test]
async fn solo_any_host_maps_to_tenant() {
    let ctx = fixtures::setup_solo(fixtures::config().expect("Failed to load config"))
        .await
        .expect("Failed to setup test");

    let _user = ctx
        .create_user(
            "test@example.com",
            &bits_domain::PasswordHash::new("hash".to_string()),
        )
        .await
        .expect("Failed to create user");

    let tenant = ctx
        .create_tenant("Solo Tenant")
        .await
        .expect("Failed to create tenant");

    // Mark this tenant as the fallback
    ctx.mark_tenant_as_fallback(tenant.id)
        .await
        .expect("Failed to mark tenant as fallback");

    // ANY host should resolve to the fallback tenant
    for host in ["example.com", "anything.com", "random.host", "localhost"] {
        let realm = bits_app::tenant::resolve_realm(&ctx.state, Scheme::Http, host).await;
        match realm {
            Realm::Creator(t) => assert_eq!(
                t.id, tenant.id,
                "Expected tenant {} for host {}, got {}",
                tenant.id, host, t.id
            ),
            _ => panic!("Expected Creator realm for host {}, got {:?}", host, realm),
        }
    }
}

#[tokio::test]
async fn solo_demo_takes_precedence() {
    let mut config = fixtures::config().expect("Failed to load config");
    config.platform_domain = "bits.page".to_string();

    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    let _user = ctx
        .create_user(
            "test@example.com",
            &bits_domain::PasswordHash::new("hash".to_string()),
        )
        .await
        .expect("Failed to create user");

    let _tenant = ctx
        .create_tenant("Solo Tenant")
        .await
        .expect("Failed to create tenant");

    // Demo subdomain should return Demo, not tenant
    let realm = bits_app::tenant::resolve_realm(&ctx.state, Scheme::Https, "jcf.bits.page").await;
    assert!(
        matches!(realm, Realm::Demo(_)),
        "Expected Demo realm for demo subdomain, got {:?}",
        realm
    );

    // Non-demo subdomain should return tenant
    let realm = bits_app::tenant::resolve_realm(&ctx.state, Scheme::Https, "other.bits.page").await;
    assert!(
        matches!(realm, Realm::Creator(_)),
        "Expected Creator realm for non-demo subdomain, got {:?}",
        realm
    );

    // Random domain should return tenant
    let realm =
        bits_app::tenant::resolve_realm(&ctx.state, Scheme::Https, "custom.example.com").await;
    assert!(
        matches!(realm, Realm::Creator(_)),
        "Expected Creator realm for custom domain, got {:?}",
        realm
    );
}

#[tokio::test]
async fn solo_no_fallback_tenant_returns_not_found() {
    let ctx = fixtures::setup_solo(fixtures::config().expect("Failed to load config"))
        .await
        .expect("Failed to setup test");

    // Clear all fallback tenants
    sqlx::query!("update tenants set is_fallback = false")
        .execute(&ctx.db_pool)
        .await
        .expect("Failed to clear fallback");

    // No fallback tenant - should return NotFound for unknown hosts
    let realm = bits_app::tenant::resolve_realm(&ctx.state, Scheme::Http, "any.host").await;
    assert!(
        matches!(realm, Realm::NotFound),
        "Expected NotFound when no fallback tenant exists, got {:?}",
        realm
    );
}

#[tokio::test]
async fn solo_multiple_tenants_returns_fallback() {
    let ctx = fixtures::setup_solo(fixtures::config().expect("Failed to load config"))
        .await
        .expect("Failed to setup test");

    let _user = ctx
        .create_user(
            "test@example.com",
            &bits_domain::PasswordHash::new("hash".to_string()),
        )
        .await
        .expect("Failed to create user");

    let tenant1 = ctx
        .create_tenant("First Tenant")
        .await
        .expect("Failed to create first tenant");

    let _tenant2 = ctx
        .create_tenant("Second Tenant")
        .await
        .expect("Failed to create second tenant");

    // Mark first tenant as fallback
    ctx.mark_tenant_as_fallback(tenant1.id)
        .await
        .expect("Failed to mark tenant as fallback");

    // Should always return the fallback tenant
    let realm = bits_app::tenant::resolve_realm(&ctx.state, Scheme::Http, "any.host").await;
    match realm {
        Realm::Creator(t) => assert_eq!(
            t.id, tenant1.id,
            "Expected fallback tenant in solo mode, got tenant {}",
            t.id
        ),
        _ => panic!(
            "Expected Creator realm with fallback tenant, got {:?}",
            realm
        ),
    }
}
