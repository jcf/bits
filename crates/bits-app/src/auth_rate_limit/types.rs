//! Type definitions for authentication rate limiting

// TODO: Replace with comprehensive PII type system
// See: .claude/prompts/20251208-pii-type-system.org
//
// Current approach: Simple newtype to distinguish email from String in type system.
// Future: Derive macro, contextual logging, partial redaction, etc.

use std::net::IpAddr;

/// Email address (PII - handle carefully)
#[derive(Clone, Debug)]
pub struct EmailAddress(String);

impl EmailAddress {
    #[must_use]
    pub fn new(email: String) -> Self {
        Self(email)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for EmailAddress {
    fn from(email: String) -> Self {
        Self::new(email)
    }
}

impl From<&str> for EmailAddress {
    fn from(email: &str) -> Self {
        Self::new(email.to_string())
    }
}

/// Visitor identity for rate limiting
pub struct Visitor {
    pub ip: IpAddr,
    pub email: EmailAddress,
}

impl Visitor {
    #[must_use]
    pub fn new(ip: IpAddr, email: EmailAddress) -> Self {
        Self { ip, email }
    }
}
