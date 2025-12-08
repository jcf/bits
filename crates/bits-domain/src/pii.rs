//! Generic PII wrapper for ad-hoc PII types.
//!
//! Use this wrapper when you need to mark a type as PII but don't want to
//! create a dedicated newtype. The wrapper provides PII-safe Debug output
//! and explicit access methods.
//!
//! # When to use
//!
//! - Prefer dedicated types (Email, Name, IpAddress) when available
//! - Use Pii<T> for one-off PII cases (phone numbers, addresses, etc.)
//! - Use Pii<T> in struct fields to mark them as PII
//!
//! # Examples
//!
//! ```
//! use bits_domain::Pii;
//!
//! struct User {
//!     id: i64,
//!     phone: Pii<String>,  // PII - will be redacted in Debug
//! }
//!
//! let user = User {
//!     id: 123,
//!     phone: Pii::new("+1-555-0100".to_string()),
//! };
//!
//! // Debug output: User { id: 123, phone: Pii(<redacted>) }
//! println!("{:?}", user);
//!
//! // Explicit access for actual use
//! let phone_str = user.phone.expose();
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;

/// Generic wrapper for Personally Identifiable Information.
///
/// Wraps any type T and provides PII-safe Debug output that redacts the value.
/// Access the inner value via `.expose()` to make PII access auditable.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Pii<T>(T);

impl<T> Pii<T> {
    /// Wrap a value as PII.
    #[must_use]
    pub fn new(value: T) -> Self {
        Pii(value)
    }

    /// Expose the inner value.
    ///
    /// This is intentionally explicit to make PII access auditable.
    /// Use this when you need to access the actual value for:
    /// - Database queries
    /// - Security logging (with appropriate context)
    /// - User-facing display
    #[must_use]
    pub fn expose(&self) -> &T {
        &self.0
    }

    /// Consume and return the inner value.
    #[must_use]
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Map the inner value to a different type.
    ///
    /// The result remains wrapped as PII.
    #[must_use]
    pub fn map<U, F>(self, f: F) -> Pii<U>
    where
        F: FnOnce(T) -> U,
    {
        Pii(f(self.0))
    }
}

// PII-safe Debug: redact the inner value regardless of type
impl<T> fmt::Debug for Pii<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Pii").field(&"<redacted>").finish()
    }
}

// Only implement Display if T implements Display
impl<T: fmt::Display> fmt::Display for Pii<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T> From<T> for Pii<T> {
    fn from(value: T) -> Self {
        Pii::new(value)
    }
}

impl<T> AsRef<T> for Pii<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_constructor() {
        let pii = Pii::new("sensitive".to_string());
        assert_eq!(pii.expose(), "sensitive");
    }

    #[test]
    fn debug_redacted() {
        let pii = Pii::new("secret data");
        let debug = format!("{:?}", pii);
        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains("secret"));
    }

    #[test]
    fn display_shows_value() {
        let pii = Pii::new("visible");
        assert_eq!(pii.to_string(), "visible");
    }

    #[test]
    fn expose_access() {
        let pii = Pii::new(42);
        assert_eq!(*pii.expose(), 42);
    }

    #[test]
    fn into_inner() {
        let pii = Pii::new(vec![1, 2, 3]);
        assert_eq!(pii.into_inner(), vec![1, 2, 3]);
    }

    #[test]
    fn map_transformation() {
        let pii = Pii::new("hello");
        let upper = pii.map(|s| s.to_uppercase());
        assert_eq!(upper.expose(), "HELLO");
    }

    #[test]
    fn from_trait() {
        let pii: Pii<i32> = 42.into();
        assert_eq!(*pii.expose(), 42);
    }

    #[test]
    fn as_ref_trait() {
        let pii = Pii::new("value".to_string());
        let s: &String = pii.as_ref();
        assert_eq!(s, "value");
    }

    #[test]
    fn debug_works_with_any_type() {
        let string_pii = Pii::new("test");
        assert!(format!("{:?}", string_pii).contains("<redacted>"));

        let int_pii = Pii::new(123);
        assert!(format!("{:?}", int_pii).contains("<redacted>"));

        let vec_pii = Pii::new(vec![1, 2, 3]);
        assert!(format!("{:?}", vec_pii).contains("<redacted>"));
    }

    #[test]
    fn serde_serialization() {
        let pii = Pii::new("secret");
        let json = serde_json::to_string(&pii).unwrap();
        assert_eq!(json, r#""secret""#);
    }

    #[test]
    fn serde_deserialization() {
        let json = r#""secret""#;
        let pii: Pii<String> = serde_json::from_str(json).unwrap();
        assert_eq!(pii.expose(), "secret");
    }

    #[derive(Debug)]
    struct User {
        #[allow(dead_code)]
        id: i64,
        phone: Pii<String>,
    }

    #[test]
    fn usage_in_struct() {
        let user = User {
            id: 123,
            phone: Pii::new("+1-555-0100".to_string()),
        };

        let debug = format!("{:?}", user);
        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains("555"));
        assert_eq!(user.phone.expose(), "+1-555-0100");
    }
}
