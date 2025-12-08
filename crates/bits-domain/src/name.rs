//! Name type with PII-safe Debug output.
//!
//! Names (first, last, full) are PII under GDPR and should be handled carefully.
//! They should not appear in general debug output but may be needed for
//! user-facing display and database operations.
//!
//! # Construction
//!
//! - `Name::new(s)` - Trusted constructor (from database, already validated)
//! - `Name::parse(s)` - Validating constructor (from user input)
//! - `s.parse::<Name>()` - Via FromStr trait

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[repr(transparent)]
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(transparent))]
pub struct Name(String);

#[derive(Debug, thiserror::Error)]
pub enum NameError {
    #[error("Name cannot be empty")]
    Empty,
    #[error("Name exceeds maximum length of {max} characters (got {actual})")]
    TooLong { max: usize, actual: usize },
}

impl Name {
    /// Maximum length for names (reasonable limit for database storage)
    pub const MAX_LENGTH: usize = 255;

    /// Trusted constructor - wraps without validation.
    ///
    /// Use for database data or already-validated strings.
    /// The string should already be normalized (trimmed).
    #[must_use]
    pub fn new(s: String) -> Self {
        Name(s)
    }

    /// Validating constructor - parses and validates user input.
    ///
    /// Trims whitespace and validates:
    /// - Not empty after trimming
    /// - Not exceeding MAX_LENGTH
    ///
    /// Use this at boundaries when accepting untrusted input.
    pub fn parse(s: &str) -> Result<Self, NameError> {
        let trimmed = s.trim();

        if trimmed.is_empty() {
            return Err(NameError::Empty);
        }

        if trimmed.len() > Self::MAX_LENGTH {
            return Err(NameError::TooLong {
                max: Self::MAX_LENGTH,
                actual: trimmed.len(),
            });
        }

        Ok(Name(trimmed.to_string()))
    }

    /// Get the name as a string slice.
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

impl FromStr for Name {
    type Err = NameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Name::parse(s)
    }
}

// PII-safe Debug: redact the name
impl fmt::Debug for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Name").field(&"<redacted>").finish()
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// For sqlx deserialization from database (trusted data)
impl From<String> for Name {
    fn from(s: String) -> Self {
        Name::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_name() {
        let name = Name::parse("Alice").unwrap();
        assert_eq!(name.as_str(), "Alice");
    }

    #[test]
    fn parse_trims_whitespace() {
        let name = Name::parse("  Bob Smith  ").unwrap();
        assert_eq!(name.as_str(), "Bob Smith");
    }

    #[test]
    fn parse_allows_unicode() {
        let name = Name::parse("José García").unwrap();
        assert_eq!(name.as_str(), "José García");
    }

    #[test]
    fn parse_rejects_empty() {
        assert!(matches!(Name::parse(""), Err(NameError::Empty)));
        assert!(matches!(Name::parse("   "), Err(NameError::Empty)));
    }

    #[test]
    fn parse_rejects_too_long() {
        let long_name = "a".repeat(Name::MAX_LENGTH + 1);
        let result = Name::parse(&long_name);
        assert!(matches!(result, Err(NameError::TooLong { .. })));
    }

    #[test]
    fn parse_accepts_max_length() {
        let max_name = "a".repeat(Name::MAX_LENGTH);
        let name = Name::parse(&max_name).unwrap();
        assert_eq!(name.as_str().len(), Name::MAX_LENGTH);
    }

    #[test]
    fn from_str_trait() {
        let name: Name = "Charlie".parse().unwrap();
        assert_eq!(name.as_str(), "Charlie");
    }

    #[test]
    fn new_trusted_constructor() {
        let name = Name::new("Diana".to_string());
        assert_eq!(name.as_str(), "Diana");
    }

    #[test]
    fn debug_redacted() {
        let name = Name::new("Secret Name".to_string());
        let debug = format!("{:?}", name);
        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains("Secret"));
    }

    #[test]
    fn display_shows_name() {
        let name = Name::new("Eve".to_string());
        assert_eq!(name.to_string(), "Eve");
    }

    #[test]
    fn as_ref_str() {
        let name = Name::new("Frank".to_string());
        let s: &str = name.as_ref();
        assert_eq!(s, "Frank");
    }

    #[test]
    fn into_inner() {
        let name = Name::new("Grace".to_string());
        assert_eq!(name.into_inner(), "Grace");
    }

    #[test]
    fn from_string() {
        let name: Name = "Henry".to_string().into();
        assert_eq!(name.as_str(), "Henry");
    }
}
