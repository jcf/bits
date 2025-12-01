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
