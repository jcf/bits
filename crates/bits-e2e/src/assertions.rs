use reqwest::Response;

/// Assert that a response contains the specified headers with exact values.
/// Only checks the headers provided - ignores other headers in the response.
pub fn assert_headers(response: &Response, expected: &[(&str, &str)]) {
    let headers = response.headers();
    for (name, expected_value) in expected {
        let actual = headers
            .get(*name)
            .unwrap_or_else(|| panic!("Missing header: {}", name))
            .to_str()
            .unwrap_or_else(|_| panic!("Header {} contains invalid characters", name));
        assert_eq!(
            actual, *expected_value,
            "Header '{}' mismatch: expected '{}', got '{}'",
            name, expected_value, actual
        );
    }
}

/// Assert that a response contains a header (value doesn't matter).
pub fn assert_header_exists(response: &Response, name: &str) {
    assert!(
        response.headers().contains_key(name),
        "Missing header: {}",
        name
    );
}
