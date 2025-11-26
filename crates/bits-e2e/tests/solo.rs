use bits_e2e::{fixtures, request, server};

#[tokio::test]
#[ignore] // Remove once implemented
async fn all_requests_routed_to_tenant() {
    let srv = server::spawn_solo().await.expect("Failed to spawn server");

    let resp = request::get(&srv, "/").send().await;

    assert_eq!(resp.status(), 200);
    // TODO: Assert no marketing content present
}

#[tokio::test]
#[ignore] // Remove once implemented
async fn no_marketing_pages() {
    let srv = server::spawn_solo().await.expect("Failed to spawn server");

    let resp = request::get(&srv, "/signup").send().await;

    // Should route to app, not marketing
    assert_ne!(resp.status(), 404);
}
