//! Password type with secret-safe Debug output and automatic memory zeroing.
//!
//! Wraps secrecy::Secret to provide type safety and automatic memory zeroing.
//! Plaintext passwords should NEVER appear in logs.
//!
//! Use `.expose_secret()` to access the inner string when needed.

use secrecy::SecretString;
use std::fmt;
use std::ops::Deref;

#[repr(transparent)]
#[derive(Clone)]
pub struct Password(SecretString);

impl Password {
    /// Create a password from user input.
    ///
    /// Use this at boundaries when accepting password input from forms.
    #[must_use]
    pub fn new(password: String) -> Self {
        Password(SecretString::from(password))
    }
}

impl Deref for Password {
    type Target = SecretString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Secret-safe Debug: delegates to Secret's redacted impl
impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Password").field(&"<redacted>").finish()
    }
}

// Secret-safe Display: redact the password
impl fmt::Display for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<redacted>")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::ExposeSecret;

    #[test]
    fn new_constructor() {
        let password = Password::new("secret123".to_string());
        assert_eq!(password.expose_secret(), "secret123");
    }

    #[test]
    fn debug_redacted() {
        let password = Password::new("secret123".to_string());
        let debug = format!("{:?}", password);
        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains("secret"));
    }

    #[test]
    fn display_redacted() {
        let password = Password::new("secret123".to_string());
        assert_eq!(password.to_string(), "<redacted>");
    }

    #[test]
    fn deref_to_secret() {
        let password = Password::new("secret123".to_string());
        assert_eq!(password.expose_secret(), "secret123");
    }
}
