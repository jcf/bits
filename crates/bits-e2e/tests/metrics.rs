use bits_e2e::{fixtures, request};

#[tokio::test]
async fn metrics_endpoint_works_without_auth() {
    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    // Make a request to ensure metrics are initialized
    let _ = request::get(&ctx, "/").send().await;

    let response = request::get(&ctx, "/metrics").send().await;

    assert_eq!(response.status(), 200);

    let body = response.text().await.expect("Failed to read body");
    assert!(
        body.contains("# TYPE"),
        "Response should contain Prometheus metrics"
    );
}

#[tokio::test]
async fn metrics_endpoint_blocks_without_token() {
    let mut config = fixtures::config().expect("Failed to load config");
    config.metrics_auth_token = Some("secret-token".to_string());

    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    let response = request::get(&ctx, "/metrics").send().await;

    assert_eq!(response.status(), 401);

    let body = response.text().await.expect("Failed to read body");
    assert_eq!(body, "Unauthorized: Invalid or missing bearer token");
}

#[tokio::test]
async fn metrics_endpoint_blocks_wrong_token() {
    let mut config = fixtures::config().expect("Failed to load config");
    config.metrics_auth_token = Some("correct-token".to_string());

    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    let response = request::get(&ctx, "/metrics")
        .header("Authorization", "Bearer wrong-token")
        .send()
        .await;

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn metrics_endpoint_allows_correct_token() {
    let mut config = fixtures::config().expect("Failed to load config");
    let token = "correct-token";
    config.metrics_auth_token = Some(token.to_string());

    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    // Make a request to ensure metrics are initialized
    let _ = request::get(&ctx, "/").send().await;

    let response = request::get(&ctx, "/metrics")
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await;

    assert_eq!(response.status(), 200);

    let body = response.text().await.expect("Failed to read body");
    assert!(body.contains("# TYPE"));
}

#[tokio::test]
async fn metrics_endpoint_records_http_requests() {
    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    // Make a request to generate metrics
    let _ = request::get(&ctx, "/").send().await;

    let response = request::get(&ctx, "/metrics").send().await;
    assert_eq!(response.status(), 200);

    let body = response.text().await.expect("Failed to read body");
    assert!(body.contains("http_requests_total"));
    assert!(body.contains("http_request_duration_ms"));
}
