use bits_app::{http::Scheme, tenant::Realm};
use bits_e2e::fixtures;

#[tokio::test]
async fn resolve_realm_returns_platform_for_apex_domain() {
    let mut config = fixtures::config().expect("Failed to load config");
    config.platform_domain = "example.com".to_string();

    let ctx = fixtures::setup_colo(config.clone())
        .await
        .expect("Failed to setup test");

    let realm = bits_app::tenant::resolve_realm(&ctx.state, Scheme::Https, "example.com").await;

    assert!(matches!(realm, Realm::Platform { .. }));
}

#[tokio::test]
async fn resolve_realm_returns_tenancy_for_subdomain() {
    let mut config = fixtures::config().expect("Failed to load config");
    config.platform_domain = "example.com".to_string();

    let ctx = fixtures::setup_colo(config.clone())
        .await
        .expect("Failed to setup test");

    let user = ctx
        .create_user("test@example.com", "hash")
        .await
        .expect("Failed to create user");

    let (tenant, _domain) = ctx
        .create_tenant_with_domain("Test Tenant", "test.example.com", user.id)
        .await
        .expect("Failed to create tenant");

    let realm =
        bits_app::tenant::resolve_realm(&ctx.state, Scheme::Https, "test.example.com").await;

    match realm {
        Realm::Creator(t) => {
            assert_eq!(t.id, tenant.id);
            assert_eq!(t.name, "Test Tenant");
        }
        _ => panic!("Expected Creator realm, got {:?}", realm),
    }
}

#[tokio::test]
async fn resolve_realm_returns_unknown_for_nonexistent_subdomain() {
    let mut config = fixtures::config().expect("Failed to load config");
    config.platform_domain = "example.com".to_string();

    let ctx = fixtures::setup_colo(config.clone())
        .await
        .expect("Failed to setup test");

    let realm =
        bits_app::tenant::resolve_realm(&ctx.state, Scheme::Https, "nonexistent.example.com").await;

    assert!(matches!(realm, Realm::NotFound));
}
