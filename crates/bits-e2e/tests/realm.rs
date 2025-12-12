use bits::{http::Scheme, tenant::Realm};
use bits_e2e::{fixtures, request};

#[tokio::test]
async fn colo_apex_domain_returns_platform() {
    let mut config = fixtures::config().expect("Failed to load config");
    config.platform_domain = "bits.page".to_string();

    let ctx = fixtures::setup_colo(config)
        .await
        .expect("Failed to setup test");

    let realm = bits::tenant::resolve_realm(&ctx.state, Scheme::Https, "bits.page").await;

    assert!(
        matches!(realm, Realm::Platform { .. }),
        "Expected Platform realm for apex domain, got {:?}",
        realm
    );
}

#[tokio::test]
async fn colo_demo_subdomain_returns_demo() {
    let mut config = fixtures::config().expect("Failed to load config");
    config.platform_domain = "bits.page".to_string();

    let ctx = fixtures::setup_colo(config)
        .await
        .expect("Failed to setup test");

    let realm = bits::tenant::resolve_realm(&ctx.state, Scheme::Https, "jcf.bits.page").await;

    assert!(
        matches!(realm, Realm::Demo(_)),
        "Expected Demo realm for demo subdomain, got {:?}",
        realm
    );
}

#[tokio::test]
async fn colo_tenant_subdomain_returns_creator() {
    let mut config = fixtures::config().expect("Failed to load config");
    config.platform_domain = "bits.page".to_string();

    let ctx = fixtures::setup_colo(config)
        .await
        .expect("Failed to setup test");

    let hash = bits_domain::PasswordHash::new("hash".to_string());
    let user = ctx
        .create_user("test@example.com", &hash)
        .await
        .expect("Failed to create user");

    let (tenant, _domain) = ctx
        .create_tenant_with_domain("Test Tenant", "test.bits.page", user.id)
        .await
        .expect("Failed to create tenant");

    let realm = bits::tenant::resolve_realm(&ctx.state, Scheme::Https, "test.bits.page").await;

    match realm {
        Realm::Creator(t) => {
            assert_eq!(t.id, tenant.id);
            assert_eq!(t.name, "Test Tenant");
        }
        _ => panic!("Expected Creator realm, got {:?}", realm),
    }
}

#[tokio::test]
async fn colo_custom_domain_returns_creator() {
    let mut config = fixtures::config().expect("Failed to load config");
    config.platform_domain = "bits.page".to_string();

    let ctx = fixtures::setup_colo(config)
        .await
        .expect("Failed to setup test");

    let hash = bits_domain::PasswordHash::new("hash".to_string());
    let user = ctx
        .create_user("test@example.com", &hash)
        .await
        .expect("Failed to create user");

    let (tenant, _domain) = ctx
        .create_tenant_with_domain("Custom Domain Tenant", "custom.example.com", user.id)
        .await
        .expect("Failed to create tenant");

    let realm = bits::tenant::resolve_realm(&ctx.state, Scheme::Https, "custom.example.com").await;

    match realm {
        Realm::Creator(t) => {
            assert_eq!(t.id, tenant.id);
            assert_eq!(t.name, "Custom Domain Tenant");
        }
        _ => panic!("Expected Creator realm for custom domain, got {:?}", realm),
    }
}

#[tokio::test]
async fn colo_unknown_subdomain_returns_not_found() {
    let mut config = fixtures::config().expect("Failed to load config");
    config.platform_domain = "bits.page".to_string();

    let ctx = fixtures::setup_colo(config)
        .await
        .expect("Failed to setup test");

    let realm =
        bits::tenant::resolve_realm(&ctx.state, Scheme::Https, "nonexistent.bits.page").await;

    assert!(
        matches!(realm, Realm::NotFound),
        "Expected NotFound for unknown subdomain, got {:?}",
        realm
    );
}

#[tokio::test]
async fn colo_unknown_custom_domain_returns_not_found() {
    let mut config = fixtures::config().expect("Failed to load config");
    config.platform_domain = "bits.page".to_string();

    let ctx = fixtures::setup_colo(config)
        .await
        .expect("Failed to setup test");

    let realm = bits::tenant::resolve_realm(&ctx.state, Scheme::Https, "unknown.example.com").await;

    assert!(
        matches!(realm, Realm::NotFound),
        "Expected NotFound for unknown custom domain, got {:?}",
        realm
    );
}

#[tokio::test]
async fn colo_nonexistent_tenant_returns_404_status() {
    let mut config = fixtures::config().expect("Failed to load config");
    config.platform_domain = "bits.page".to_string();

    let ctx = fixtures::setup_colo(config)
        .await
        .expect("Failed to setup test");

    let response = request::get(&ctx, "/")
        .header("Host", "nonexistent.bits.page")
        .send()
        .await;

    assert_eq!(
        response.status(),
        404,
        "Non-existent tenant should return 404 status"
    );
}
