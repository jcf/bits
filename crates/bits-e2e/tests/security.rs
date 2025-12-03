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
async fn csrf_blocks_requests_without_token() {
    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    let response = request::post(&ctx, "/api/sessions").send().await;

    assert_eq!(response.status(), 403);
    let body = response.text().await.expect("Failed to read body");
    assert_eq!(body, "Forbidden.\n");
}

#[tokio::test]
async fn csrf_allows_requests_with_valid_token() {
    use scraper::{Html, Selector};

    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    // Create a client with cookie jar to maintain session
    let client = request::cookie_client();
    let base_url = ctx.server.url("/");

    // Load home page to get CSRF token and establish session
    let response = client
        .get(&base_url)
        .send()
        .await
        .expect("Failed to load home page");
    assert_eq!(response.status(), 200);

    // Parse HTML to extract CSRF token from meta tag
    let html = response.text().await.expect("Failed to read HTML");
    let document = Html::parse_document(&html);
    let selector = Selector::parse("meta[name='csrf-token']").unwrap();
    let csrf_token = document
        .select(&selector)
        .next()
        .and_then(|el| el.value().attr("content"))
        .expect("CSRF token not found in page");

    // Make POST request with CSRF token and requested-with header for analytics
    let login_url = format!("{}/api/sessions", base_url.trim_end_matches('/'));
    let response = client
        .post(&login_url)
        .header("csrf-token", csrf_token)
        .header("requested-with", "bits/test")
        .header("content-type", "application/x-www-form-urlencoded")
        .body("email=test@example.com&password=invalid")
        .send()
        .await
        .expect("Failed to send POST request");

    // Should pass CSRF check (auth will fail with invalid credentials)
    assert_eq!(response.status(), 401, "Expected auth failure, not CSRF rejection");
}

#[tokio::test]
async fn csrf_rejects_invalid_token() {
    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    // Create a client with cookie jar
    let client = request::cookie_client();
    let base_url = ctx.server.url("/");

    // Load home page to establish session (but we'll use wrong token)
    let response = client
        .get(&base_url)
        .send()
        .await
        .expect("Failed to load home page");
    assert_eq!(response.status(), 200);

    // Make POST request with invalid CSRF token
    let login_url = format!("{}/api/sessions", base_url.trim_end_matches('/'));
    let response = client
        .post(&login_url)
        .header("csrf-token", "invalid_token_here")
        .header("requested-with", "bits/test")
        .header("content-type", "application/x-www-form-urlencoded")
        .body("email=test@example.com&password=invalid")
        .send()
        .await
        .expect("Failed to send POST request");

    assert_eq!(response.status(), 403);
}

#[tokio::test]
async fn csrf_allows_token_in_form_body() {
    use scraper::{Html, Selector};

    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    let client = request::cookie_client();
    let base_url = ctx.server.url("/");

    // Load home page to get CSRF token
    let response = client
        .get(&base_url)
        .send()
        .await
        .expect("Failed to load home page");
    assert_eq!(response.status(), 200);

    let html = response.text().await.expect("Failed to read HTML");
    let document = Html::parse_document(&html);
    let selector = Selector::parse("meta[name='csrf-token']").unwrap();
    let csrf_token = document
        .select(&selector)
        .next()
        .and_then(|el| el.value().attr("content"))
        .expect("CSRF token not found");

    // Make POST request with token in form body (not header)
    let login_url = format!("{}/api/sessions", base_url.trim_end_matches('/'));
    let body = format!(
        "csrf_token={}&email=test@example.com&password=invalid",
        urlencoding::encode(csrf_token)
    );
    let response = client
        .post(&login_url)
        .header("requested-with", "bits/test")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to send POST request");

    // Should pass CSRF check (auth will fail with invalid credentials)
    assert_eq!(response.status(), 401, "Expected auth failure, not CSRF rejection");
}

#[tokio::test]
async fn csrf_supports_multi_tab_usage() {
    use scraper::{Html, Selector};

    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    let client = request::cookie_client();
    let base_url = ctx.server.url("/");

    // Load home page to get CSRF token (simulating tab 1)
    let response = client.get(&base_url).send().await.expect("Failed to load home page");
    let html = response.text().await.expect("Failed to read HTML");
    let document = Html::parse_document(&html);
    let selector = Selector::parse("meta[name='csrf-token']").unwrap();
    let csrf_token = document
        .select(&selector)
        .next()
        .and_then(|el| el.value().attr("content"))
        .expect("CSRF token not found");

    // First request with token (simulating form submit from tab 1)
    let login_url = format!("{}/api/sessions", base_url.trim_end_matches('/'));
    let response1 = client
        .post(&login_url)
        .header("csrf-token", csrf_token)
        .header("requested-with", "bits/test")
        .header("content-type", "application/x-www-form-urlencoded")
        .body("email=test@example.com&password=invalid")
        .send()
        .await
        .expect("Failed to send first POST");
    assert_eq!(response1.status(), 401, "First tab: expected auth failure, not CSRF rejection");

    // Second request with same token (simulating form submit from tab 2)
    // This should work because we use session-lifetime tokens
    let response2 = client
        .post(&login_url)
        .header("csrf-token", csrf_token)
        .header("requested-with", "bits/test")
        .header("content-type", "application/x-www-form-urlencoded")
        .body("email=test@example.com&password=invalid")
        .send()
        .await
        .expect("Failed to send second POST");
    assert_eq!(response2.status(), 401, "Second tab: expected auth failure, not CSRF rejection (multi-tab should work)");
}

#[tokio::test]
async fn csrf_rejects_cross_session_tokens() {
    use scraper::{Html, Selector};

    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    // Client 1: Get CSRF token in session A
    let client1 = request::cookie_client();
    let base_url = ctx.server.url("/");
    let response = client1.get(&base_url).send().await.expect("Failed to load page");
    let html = response.text().await.expect("Failed to read HTML");
    let document = Html::parse_document(&html);
    let selector = Selector::parse("meta[name='csrf-token']").unwrap();
    let token_from_session_a = document
        .select(&selector)
        .next()
        .and_then(|el| el.value().attr("content"))
        .expect("CSRF token not found");

    // Client 2: Different session (session B)
    let client2 = request::cookie_client();
    // Load page to establish session B (but we'll use token from session A)
    let _ = client2.get(&base_url).send().await.expect("Failed to load page");

    // Try to use token from session A in session B
    let login_url = format!("{}/api/sessions", base_url.trim_end_matches('/'));
    let response = client2
        .post(&login_url)
        .header("csrf-token", token_from_session_a)
        .header("requested-with", "bits/test")
        .header("content-type", "application/x-www-form-urlencoded")
        .body("email=test@example.com&password=invalid")
        .send()
        .await
        .expect("Failed to send POST");

    assert_eq!(response.status(), 403, "Cross-session token should be rejected");
}

#[tokio::test]
async fn csrf_allows_get_requests_without_token() {
    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    // GET requests should not require CSRF token
    let response = request::get(&ctx, "/").send().await;
    assert_eq!(response.status(), 200);

    // Also test with API endpoint
    let response = request::get(&ctx, "/api/session").send().await;
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn csrf_blocks_post_from_fresh_session() {
    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    let client = request::cookie_client();
    let base_url = ctx.server.url("/");

    // POST without ever loading a page (no CSRF token in DB)
    let login_url = format!("{}/api/sessions", base_url.trim_end_matches('/'));
    let response = client
        .post(&login_url)
        .header("requested-with", "bits/test")
        .header("content-type", "application/x-www-form-urlencoded")
        .body("email=test@example.com&password=invalid")
        .send()
        .await
        .expect("Failed to send POST");

    assert_eq!(response.status(), 403, "POST without page load should be blocked");
}

fn get_request_id(response: &reqwest::Response) -> &str {
    response
        .headers()
        .get("x-request-id")
        .expect("Missing x-request-id")
        .to_str()
        .expect("Invalid x-request-id")
}

#[tokio::test]
async fn request_id_generated_for_requests() {
    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    let r1 = request::get(&ctx, "/").send().await;
    let r2 = request::get(&ctx, "/").send().await;

    assert_eq!(r1.status(), 200);
    assert_eq!(r2.status(), 200);

    assert_ne!(get_request_id(&r1), get_request_id(&r2));
}
