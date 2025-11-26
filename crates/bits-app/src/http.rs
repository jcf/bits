use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scheme {
    Http,
    Https,
    Unsupported,
}

impl FromStr for Scheme {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "http" => Scheme::Http,
            "https" => Scheme::Https,
            _ => Scheme::Unsupported,
        })
    }
}

impl Scheme {
    fn default_port(&self) -> Option<u16> {
        match self {
            Scheme::Http => Some(80),
            Scheme::Https => Some(443),
            Scheme::Unsupported => None,
        }
    }
}

pub fn normalize_host(scheme: Scheme, host: &str) -> String {
    if let Some(default_port) = scheme.default_port() {
        if let Some(h) = host.strip_suffix(&format!(":{}", default_port)) {
            return h.to_string();
        }
    }
    host.to_string()
}

#[cfg(feature = "server")]
pub fn extract_scheme<B>(req: &dioxus::server::axum::http::Request<B>) -> Scheme {
    let scheme_str = req
        .headers()
        .get("x-forwarded-proto")
        .and_then(|h| h.to_str().ok())
        .or_else(|| req.uri().scheme_str())
        .unwrap_or("https");

    scheme_str.parse().unwrap_or(Scheme::Unsupported)
}

#[cfg(feature = "server")]
pub fn extract_host<B>(req: &dioxus::server::axum::http::Request<B>) -> Option<String> {
    req.headers()
        .get("host")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("http", Scheme::Http)]
    #[case("HTTP", Scheme::Http)]
    #[case("https", Scheme::Https)]
    #[case("HTTPS", Scheme::Https)]
    #[case("ftp", Scheme::Unsupported)]
    #[case("", Scheme::Unsupported)]
    fn parse_scheme(#[case] input: &str, #[case] expected: Scheme) {
        assert_eq!(input.parse::<Scheme>(), Ok(expected));
    }

    #[rstest]
    #[case(Scheme::Https, "example.com:443", "example.com")]
    #[case(Scheme::Https, "example.com:8443", "example.com:8443")]
    #[case(Scheme::Https, "example.com:80", "example.com:80")]
    #[case(Scheme::Http, "example.com:80", "example.com")]
    #[case(Scheme::Http, "example.com:8080", "example.com:8080")]
    #[case(Scheme::Http, "example.com:443", "example.com:443")]
    #[case(Scheme::Https, "example.com", "example.com")]
    #[case(Scheme::Http, "example.com", "example.com")]
    #[case(Scheme::Unsupported, "example.com:443", "example.com:443")]
    #[case(Scheme::Https, "jcf.bits.page:443", "jcf.bits.page")]
    fn normalize(#[case] scheme: Scheme, #[case] input: &str, #[case] expected: &str) {
        assert_eq!(normalize_host(scheme, input), expected);
    }
}
