//! Validated email address type with PII-safe Debug output.
//!
//! The Email type provides:
//! - RFC 6531 validation via Garde (internationalized email addresses)
//! - Automatic normalization (lowercase, trimmed) during parsing
//! - PII-safe Debug output (redacted)
//! - Explicit access via `.as_str()` for auditability
//!
//! # Construction
//!
//! - `Email::new(s)` - Trusted constructor (database data, already validated)
//! - `Email::parse(s)` - Validating constructor (user input, untrusted data)
//! - `s.parse::<Email>()` - Via FromStr trait

use garde::Validate;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[repr(transparent)]
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(transparent))]
pub struct Email(String);

#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    #[error("Invalid email format: {0}")]
    Invalid(#[from] garde::Report),
}

impl Email {
    /// Trusted constructor - wraps without validation.
    ///
    /// Use for database data or already-validated strings.
    /// The string should already be normalized (lowercase, trimmed).
    #[must_use]
    pub fn new(s: String) -> Self {
        Email(s)
    }

    /// Validating constructor - parses and validates user input.
    ///
    /// Normalizes (trims and lowercases) and validates according to
    /// RFC 6531 (internationalized email addresses) via Garde.
    ///
    /// Use this at boundaries when accepting untrusted input.
    pub fn parse(s: &str) -> Result<Self, EmailError> {
        let normalized = s.trim().to_lowercase();

        // Validate using garde's email validator
        #[derive(Validate)]
        struct EmailValidator {
            #[garde(email)]
            email: String,
        }

        EmailValidator {
            email: normalized.clone(),
        }
        .validate()?;
        Ok(Email(normalized))
    }

    /// Get the email address as a string slice.
    ///
    /// This is intentionally explicit to make PII access auditable.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume and return the inner String.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl FromStr for Email {
    type Err = EmailError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Email::parse(s)
    }
}

// PII-safe Debug: redact the email address
impl fmt::Debug for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Email").field(&"<redacted>").finish()
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// For sqlx deserialization from database (trusted data)
impl From<String> for Email {
    fn from(s: String) -> Self {
        Email::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_email() {
        let email = Email::parse("user@example.com").unwrap();
        assert_eq!(email.as_str(), "user@example.com");
    }

    #[test]
    fn parse_normalizes_to_lowercase() {
        let email = Email::parse("User@Example.COM").unwrap();
        assert_eq!(email.as_str(), "user@example.com");
    }

    #[test]
    fn parse_trims_whitespace() {
        let email = Email::parse("  user@example.com  ").unwrap();
        assert_eq!(email.as_str(), "user@example.com");
    }

    #[test]
    fn parse_rejects_no_at_sign() {
        assert!(Email::parse("userexample.com").is_err());
    }

    #[test]
    fn parse_rejects_no_domain() {
        assert!(Email::parse("user@").is_err());
    }

    #[test]
    fn parse_rejects_invalid_format() {
        assert!(Email::parse("not an email").is_err());
    }

    #[test]
    fn from_str_trait() {
        let email: Email = "user@example.com".parse().unwrap();
        assert_eq!(email.as_str(), "user@example.com");
    }

    #[test]
    fn new_trusted_constructor() {
        let email = Email::new("user@example.com".to_string());
        assert_eq!(email.as_str(), "user@example.com");
    }

    #[test]
    fn debug_redacted() {
        let email = Email::new("user@example.com".to_string());
        let debug = format!("{:?}", email);
        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains("user@example.com"));
    }

    #[test]
    fn display_shows_email() {
        let email = Email::new("user@example.com".to_string());
        assert_eq!(email.to_string(), "user@example.com");
    }

    #[test]
    fn as_ref_str() {
        let email = Email::new("user@example.com".to_string());
        let s: &str = email.as_ref();
        assert_eq!(s, "user@example.com");
    }

    #[test]
    fn into_inner() {
        let email = Email::new("user@example.com".to_string());
        assert_eq!(email.into_inner(), "user@example.com");
    }
}
