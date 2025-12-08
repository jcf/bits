#[cfg(feature = "server")]
use sqlx::PgPool;
use std::fmt;

/// Reserved system names (compile-time perfect hash)
mod reserved {
    use phf::phf_set;

    pub static NAMES: phf::Set<&'static str> = phf_set! {
        // Infrastructure
        "api",
        "app",
        "assets",
        "cdn",
        "download",
        "downloads",
        "files",
        "images",
        "img",
        "media",
        "static",
        "uploads",
        "www",
        "www1",
        "www2",

        // Auth & accounts
        "account",
        "accounts",
        "auth",
        "login",
        "logout",
        "oauth",
        "preferences",
        "profile",
        "register",
        "settings",
        "signin",
        "signout",
        "signup",


        // Admin & support
        "abuse",
        "admin",
        "administrator",
        "contact",
        "dmca",
        "feedback",
        "help",
        "legal",
        "mod",
        "moderator",
        "privacy",
        "root",
        "safety",
        "security",
        "superuser",
        "support",
        "terms",
        "tos",
        "trust",

        // Billing & business
        "billing",
        "checkout",
        "enterprise",
        "invoice",
        "pay",
        "payment",
        "payments",
        "plans",
        "pricing",
        "subscribe",
        "subscription",

        // Technical
        "alpha",
        "analytics",
        "beta",
        "canary",
        "demo",
        "dev",
        "development",
        "email",
        "ftp",
        "git",
        "health",
        "imap",
        "mail",
        "mailer",
        "metrics",
        "ping",
        "pop",
        "preview",
        "prod",
        "production",
        "sandbox",
        "sftp",
        "smtp",
        "ssh",
        "staging",
        "stats",
        "status",
        "test",
        "testing",

        // Services
        "blog",
        "community",
        "console",
        "dashboard",
        "discover",
        "docs",
        "documentation",
        "explore",
        "forum",
        "marketplace",
        "news",
        "portal",
        "search",
        "shop",
        "store",
        "wiki",

        // Brand protection
        "bits",
        "bitsapp",
        "bitshq",
        "employee",
        "getbits",
        "invetica",
        "mybits",
        "official",
        "staff",
        "team",
        "usebits",
        "verified",

        // Protocol prefixes
        "http",
        "https",
        "localhost",
        "svn",

        // Misc reserved
        "anonymous",
        "create",
        "creator",
        "creators",
        "delete",
        "edit",
        "guest",
        "me",
        "member",
        "members",
        "my",
        "new",
        "nil",
        "none",
        "null",
        "self",
        "this",
        "undefined",
        "user",
        "users",
        "void",
    };
}

/// Configuration for handle validation
#[derive(Debug, Clone)]
pub struct HandleConfig {
    pub min_length: usize,
    pub max_length: usize,
    pub check_reserved: bool,
}

impl Default for HandleConfig {
    fn default() -> Self {
        Self {
            min_length: 3,
            max_length: 32,
            check_reserved: true,
        }
    }
}

/// A validated subdomain handle (e.g., "jcf" in "jcf.bits.page")
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Handle(String);

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, serde::Serialize, serde::Deserialize)]
pub enum HandleError {
    #[error("handle must be at least {min} characters (got {actual})")]
    TooShort { min: usize, actual: usize },

    #[error("handle must be at most {max} characters (got {actual})")]
    TooLong { max: usize, actual: usize },

    #[error("handle must start with a letter")]
    MustStartWithLetter,

    #[error("handle cannot end with a hyphen")]
    CannotEndWithHyphen,

    #[error("handle cannot contain consecutive hyphens")]
    ConsecutiveHyphens,

    #[error("handle contains invalid character '{0}' at position {1}")]
    InvalidCharacter(char, usize),

    #[error("'{0}' is a reserved name")]
    Reserved(String),
}

impl Handle {
    /// Create a new handle with validation
    pub fn new(input: impl Into<String>) -> Result<Self, HandleError> {
        Self::with_config(input, &HandleConfig::default())
    }

    /// Create a new handle with custom configuration
    pub fn with_config(
        input: impl Into<String>,
        config: &HandleConfig,
    ) -> Result<Self, HandleError> {
        let normalized = input.into().trim().to_ascii_lowercase();

        // Length checks
        if normalized.len() < config.min_length {
            return Err(HandleError::TooShort {
                min: config.min_length,
                actual: normalized.len(),
            });
        }
        if normalized.len() > config.max_length {
            return Err(HandleError::TooLong {
                max: config.max_length,
                actual: normalized.len(),
            });
        }

        // Must start with letter
        let first = normalized.chars().next().unwrap(); // safe: len >= min_length
        if !first.is_ascii_lowercase() {
            return Err(HandleError::MustStartWithLetter);
        }

        // Cannot end with hyphen
        if normalized.ends_with('-') {
            return Err(HandleError::CannotEndWithHyphen);
        }

        // Character validation + consecutive hyphen check
        let mut prev_hyphen = false;
        for (i, c) in normalized.chars().enumerate() {
            match c {
                'a'..='z' | '0'..='9' => {
                    prev_hyphen = false;
                }
                '-' => {
                    if prev_hyphen {
                        return Err(HandleError::ConsecutiveHyphens);
                    }
                    prev_hyphen = true;
                }
                _ => {
                    return Err(HandleError::InvalidCharacter(c, i));
                }
            }
        }

        // Reserved names
        if config.check_reserved && reserved::NAMES.contains(&normalized) {
            return Err(HandleError::Reserved(normalized));
        }

        Ok(Handle(normalized))
    }

    /// Create from trusted source (e.g., database) without validation
    #[must_use]
    pub fn from_trusted(s: String) -> Self {
        Handle(s)
    }

    /// Get the validated handle as a string slice
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume and return the inner String
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Display for Handle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Handle {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Check if a name is reserved (system use)
#[must_use]
pub fn is_reserved(name: &str) -> bool {
    reserved::NAMES.contains(&name.to_ascii_lowercase())
}

/// A validated tenant.
///
/// SECURITY: This struct is serialized in public Realm API responses.
/// Only add fields that are safe to expose publicly (no emails, secrets, payment info, etc).
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Tenant {
    pub id: i64,
    pub name: String,
}

/// Realm represents the context for a request in multi-tenant mode
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum Realm {
    /// Platform realm - the apex domain
    Platform { domain: String },
    /// Creator realm - a creator's site
    Creator(Tenant),
    /// Demo realm - a demo profile
    Demo(Handle),
    /// Not found - subdomain doesn't match any tenant or demo
    NotFound,
}

#[cfg(feature = "server")]
async fn load_tenant_by_domain(pool: &PgPool, domain: &str) -> Result<Option<Tenant>, sqlx::Error> {
    sqlx::query_as!(
        Tenant,
        "select
           t.id,
           t.name
         from tenants t
         join tenant_domains td on td.tenant_id = t.id
         where td.domain = $1
         and td.valid_to = 'infinity'",
        domain,
    )
    .fetch_optional(pool)
    .await
}

#[cfg(feature = "server")]
pub async fn resolve_realm(
    state: &crate::AppState,
    scheme: crate::http::Scheme,
    host: &str,
) -> Realm {
    let normalized_host = crate::http::normalize_host(scheme, host);

    // Check colo-specific routing (platform domain and subdomains)
    #[cfg(feature = "colo")]
    if let Some(realm) = resolve_colo_realm(state, &normalized_host).await {
        return realm;
    }

    // Check for custom domain tenant (both colo and solo)
    match load_tenant_by_domain(&state.db, &normalized_host).await {
        Ok(Some(tenant)) => return Realm::Creator(tenant),
        Ok(None) => {}
        Err(e) => {
            tracing::error!(
                domain = %normalized_host,
                error = %e,
                "Failed to load tenant from database"
            );
        }
    }

    // Default realm when no tenant found
    Realm::NotFound
}

/// Resolve realm for colo mode (platform domain and subdomains)
#[cfg(all(feature = "server", feature = "colo"))]
async fn resolve_colo_realm(state: &crate::AppState, normalized_host: &str) -> Option<Realm> {
    let platform_domain = &state.config.platform_domain;

    // Exact match: apex domain
    if normalized_host == *platform_domain {
        return Some(Realm::Platform {
            domain: platform_domain.clone(),
        });
    }

    // Must be a subdomain of platform domain, otherwise None
    let subdomain = normalized_host.strip_suffix(&format!(".{}", platform_domain))?;
    let handle = Handle::from_trusted(subdomain.to_string());

    // Check if demo (no DB hit!)
    if crate::demos::SUBDOMAINS.contains(&handle.as_str()) {
        return Some(Realm::Demo(handle));
    }

    // Check database for real tenants
    match load_tenant_by_domain(&state.db, normalized_host).await {
        Ok(Some(tenant)) => return Some(Realm::Creator(tenant)),
        Ok(None) => {}
        Err(e) => {
            tracing::error!(
                domain = %normalized_host,
                error = %e,
                "Failed to load tenant from database"
            );
        }
    }

    Some(Realm::NotFound)
}

#[cfg(feature = "server")]
impl<S> axum_core::extract::FromRequestParts<S> for Realm
where
    S: Send + Sync,
{
    type Rejection = (dioxus::server::axum::http::StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut dioxus::server::axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts.extensions.get::<Realm>().cloned().ok_or((
            dioxus::server::axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Realm not set",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_handles() {
        let valid = [
            "alice",
            "bob123",
            "my-cool-handle",
            "a1b2c3",
            "creator-name",
        ];
        for h in valid {
            assert!(Handle::new(h).is_ok(), "expected '{}' to be valid", h);
        }
    }

    #[test]
    fn rejects_too_short() {
        assert!(matches!(
            Handle::new("ab"),
            Err(HandleError::TooShort { .. })
        ));
        assert!(matches!(
            Handle::new("a"),
            Err(HandleError::TooShort { .. })
        ));
        assert!(matches!(Handle::new(""), Err(HandleError::TooShort { .. })));
    }

    #[test]
    fn rejects_too_long() {
        let long = "a".repeat(33);
        assert!(matches!(
            Handle::new(&long),
            Err(HandleError::TooLong { .. })
        ));
    }

    #[test]
    fn rejects_invalid_start() {
        assert!(matches!(
            Handle::new("123abc"),
            Err(HandleError::MustStartWithLetter)
        ));
        assert!(matches!(
            Handle::new("-abc"),
            Err(HandleError::MustStartWithLetter)
        ));
        assert!(matches!(
            Handle::new("_abc"),
            Err(HandleError::MustStartWithLetter)
        ));
    }

    #[test]
    fn rejects_hyphen_end() {
        assert!(matches!(
            Handle::new("abc-"),
            Err(HandleError::CannotEndWithHyphen)
        ));
    }

    #[test]
    fn rejects_consecutive_hyphens() {
        assert!(matches!(
            Handle::new("a--b"),
            Err(HandleError::ConsecutiveHyphens)
        ));
        assert!(matches!(
            Handle::new("test---handle"),
            Err(HandleError::ConsecutiveHyphens)
        ));
    }

    #[test]
    fn rejects_invalid_characters() {
        // Characters that fail in the validation loop (not at start)
        assert!(matches!(
            Handle::new("hello_world"),
            Err(HandleError::InvalidCharacter('_', _))
        ));
        assert!(matches!(
            Handle::new("hello.world"),
            Err(HandleError::InvalidCharacter('.', _))
        ));
        assert!(matches!(
            Handle::new("hello@world"),
            Err(HandleError::InvalidCharacter('@', _))
        ));
        assert!(matches!(
            Handle::new("café"),
            Err(HandleError::InvalidCharacter('é', _))
        ));
        // über fails because ü is not ASCII lowercase (fails at start check)
        assert!(matches!(
            Handle::new("über"),
            Err(HandleError::MustStartWithLetter)
        ));
    }

    #[test]
    fn rejects_reserved() {
        let reserved = ["admin", "api", "www", "login", "bits", "ADMIN", "Api"];
        for h in reserved {
            assert!(
                matches!(Handle::new(h), Err(HandleError::Reserved(_))),
                "expected '{}' to be rejected as reserved",
                h
            );
        }
    }

    #[test]
    fn normalizes_to_lowercase() {
        let handle = Handle::new("AlIcE").unwrap();
        assert_eq!(handle.as_str(), "alice");
    }

    #[test]
    fn trims_whitespace() {
        let handle = Handle::new("  alice  ").unwrap();
        assert_eq!(handle.as_str(), "alice");
    }

    #[test]
    fn custom_config() {
        let config = HandleConfig {
            min_length: 5,
            max_length: 10,
            check_reserved: false,
        };
        assert!(Handle::with_config("abc", &config).is_err());
        assert!(Handle::with_config("abcde", &config).is_ok());
        assert!(Handle::with_config("abcdefghijk", &config).is_err()); // 11 chars
    }

    #[test]
    fn into_inner() {
        let handle = Handle::new("alice").unwrap();
        assert_eq!(handle.into_inner(), "alice");
    }

    #[test]
    fn is_reserved_function() {
        assert!(is_reserved("admin"));
        assert!(is_reserved("ADMIN"));
        assert!(is_reserved("Admin"));
        assert!(!is_reserved("alice"));
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    // Strategy to generate valid handle strings (no consecutive hyphens, no trailing hyphen)
    fn valid_handle_string() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                3 => "[a-z0-9]".prop_map(|s: String| s),
                1 => Just("-".to_string()),
            ],
            3..=32,
        )
        .prop_map(|parts| parts.join(""))
        .prop_filter("must start with letter", |s| {
            s.chars()
                .next()
                .map(|c| c.is_ascii_lowercase())
                .unwrap_or(false)
        })
        .prop_filter("must end with letter or digit", |s| !s.ends_with('-'))
        .prop_filter("no consecutive hyphens", |s| !s.contains("--"))
    }

    proptest! {
        #[test]
        fn handle_roundtrips_through_display(s in valid_handle_string()) {
            if let Ok(handle) = Handle::new(&s) {
                assert_eq!(handle.to_string(), s.to_lowercase());
            }
        }

        #[test]
        fn handle_rejects_invalid_lengths(s in "[a-z][a-z0-9]{0,1}") {
            assert!(Handle::new(s).is_err());
        }

        #[test]
        fn handle_normalizes_case(s in "[A-Z][a-zA-Z0-9]{2,30}") {
            if let Ok(handle) = Handle::new(&s) {
                assert_eq!(handle.as_str(), s.to_lowercase());
            }
        }
    }
}
