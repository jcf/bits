use bits_e2e::{fixtures, request};

#[tokio::test]
#[ignore]
async fn all_requests_routed_to_tenant() {
    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config).await.expect("Failed to setup test");

    let resp = request::get(&ctx, "/").send().await;

    assert_eq!(resp.status(), 200);
}

#[tokio::test]
#[ignore]
async fn no_marketing_pages() {
    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config).await.expect("Failed to setup test");

    let resp = request::get(&ctx, "/").send().await;

    // In solo mode, the root should return 200, not 404 (no marketing site)
    assert_eq!(resp.status(), 200);
}
