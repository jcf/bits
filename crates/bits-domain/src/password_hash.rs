//! Password hash type with secret-safe Debug output and automatic memory zeroing.
//!
//! Wraps secrecy::Secret to provide type safety and automatic memory zeroing.
//! Password hashes should never appear in logs, even though they're hashed.
//!
//! Use `.expose_secret()` to access the inner string when needed.

use secrecy::SecretString;
use std::fmt;
use std::ops::Deref;

#[repr(transparent)]
#[derive(Clone)]
pub struct PasswordHash(SecretString);

impl PasswordHash {
    /// Trusted constructor - wraps a hash without validation.
    ///
    /// Use when you have an already-computed hash (from database or password service).
    #[must_use]
    pub fn new(hash: String) -> Self {
        PasswordHash(SecretString::from(hash))
    }
}

impl Deref for PasswordHash {
    type Target = SecretString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Secret-safe Debug: redact the hash
impl fmt::Debug for PasswordHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("PasswordHash").field(&"<redacted>").finish()
    }
}

impl fmt::Display for PasswordHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<redacted>")
    }
}

// Manual SQLx implementation since Secret<String> doesn't derive sqlx::Type
#[cfg(feature = "sqlx")]
impl sqlx::Type<sqlx::Postgres> for PasswordHash {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

#[cfg(feature = "sqlx")]
impl<'q> sqlx::Encode<'q, sqlx::Postgres> for PasswordHash {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        use secrecy::ExposeSecret;
        <&str as sqlx::Encode<sqlx::Postgres>>::encode(self.expose_secret(), buf)
    }
}

#[cfg(feature = "sqlx")]
impl<'r> sqlx::Decode<'r, sqlx::Postgres> for PasswordHash {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let s = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        Ok(PasswordHash::new(s))
    }
}

// For sqlx deserialization from database (trusted data)
impl From<String> for PasswordHash {
    fn from(s: String) -> Self {
        PasswordHash::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::ExposeSecret;

    #[test]
    fn new_trusted_constructor() {
        let hash = PasswordHash::new("$argon2id$v=19$...".to_string());
        assert_eq!(hash.expose_secret(), "$argon2id$v=19$...");
    }

    #[test]
    fn debug_redacted() {
        let hash = PasswordHash::new("$argon2id$v=19$secret".to_string());
        let debug = format!("{:?}", hash);
        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains("secret"));
    }

    #[test]
    fn display_redacted() {
        let hash = PasswordHash::new("$argon2id$v=19$secret".to_string());
        assert_eq!(hash.to_string(), "<redacted>");
    }

    #[test]
    fn deref_to_secret() {
        let hash = PasswordHash::new("$argon2id$v=19$...".to_string());
        assert_eq!(hash.expose_secret(), "$argon2id$v=19$...");
    }
}
