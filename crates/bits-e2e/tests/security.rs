use bits_app::CspMode;
use bits_e2e::{assertions, fixtures, request};
use rstest::rstest;

#[rstest]
#[case(CspMode::Strict)]
#[case(CspMode::Development)]
#[tokio::test]
async fn solo_has_security_headers(#[case] mode: CspMode) {
    let mut config = fixtures::config().expect("Failed to load config");
    config.dangerously_allow_javascript_evaluation = matches!(mode, CspMode::Development);

    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    let response = request::get(&ctx, "/").send().await;

    assert_eq!(response.status(), 200);

    let expected_csp = bits_app::http::csp_header(mode);
    assertions::assert_headers(
        &response,
        &[
            ("content-security-policy", expected_csp.as_str()),
            ("referrer-policy", "strict-origin"),
            ("server", "bits"),
            (
                "strict-transport-security",
                "max-age=31536000; includeSubdomains",
            ),
            ("x-content-type-options", "nosniff"),
            ("x-download-options", "noopen"),
            ("x-frame-options", "DENY"),
            ("x-permitted-cross-domain-policies", "none"),
            ("x-xss-protection", "1; mode=block"),
        ],
    );
}

#[tokio::test]
async fn csrf_blocks_requests_without_header() {
    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    let response = request::post(&ctx, "/auth").send().await;

    assert_eq!(response.status(), 403);
    let body = response.text().await.expect("Failed to read body");
    assert_eq!(body, "Forbidden.\n");
}

#[tokio::test]
async fn csrf_allows_requests_with_valid_header() {
    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    let response = request::post(&ctx, "/auth")
        .header("requested-with", "bits/test-version")
        .send()
        .await;

    assert_ne!(response.status(), 403);
}

#[tokio::test]
async fn csrf_rejects_invalid_header_format() {
    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    let response = request::post(&ctx, "/auth")
        .header("requested-with", "invalid-format")
        .send()
        .await;

    assert_eq!(response.status(), 403);
}

#[tokio::test]
async fn request_id_generated_for_requests() {
    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    let response = request::get(&ctx, "/").send().await;

    assert!(
        response.headers().contains_key("x-request-id"),
        "Response should contain x-request-id header"
    );

    let request_id = response
        .headers()
        .get("x-request-id")
        .expect("x-request-id header missing")
        .to_str()
        .expect("x-request-id not valid string");

    assert!(!request_id.is_empty(), "Request ID should not be empty");
    assert_eq!(
        request_id.len(),
        36,
        "Request ID should be a UUID (36 characters)"
    );
}
