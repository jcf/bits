//! Domain ID types with PII-safe Debug output.
//!
//! All ID types redact their values in Debug output to prevent accidental logging of PII.
//! Use `.get()` or `Display` for explicit access.

use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! define_id {
    ($name:ident) => {
        #[repr(transparent)]
        #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
        #[cfg_attr(feature = "sqlx", sqlx(transparent))]
        pub struct $name(i64);

        impl $name {
            #[must_use]
            pub fn new(id: i64) -> Self {
                Self(id)
            }

            #[must_use]
            pub fn get(&self) -> i64 {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self(0)
            }
        }

        // PII-safe Debug: redact the actual ID value
        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_tuple(stringify!($name))
                    .field(&"<redacted>")
                    .finish()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<i64> for $name {
            fn from(id: i64) -> Self {
                Self(id)
            }
        }

        impl From<$name> for i64 {
            fn from(id: $name) -> Self {
                id.0
            }
        }
    };
}

define_id!(UserId);
define_id!(EmailAddressId);
define_id!(TenantId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_id_roundtrip() {
        let id = UserId::new(42);
        assert_eq!(id.get(), 42);
        assert_eq!(i64::from(id), 42);
    }

    #[test]
    fn user_id_from_i64() {
        let id: UserId = 42.into();
        assert_eq!(id.get(), 42);
    }

    #[test]
    fn user_id_display() {
        let id = UserId::new(42);
        assert_eq!(id.to_string(), "42");
    }

    #[test]
    fn user_id_debug_redacted() {
        let id = UserId::new(42);
        let debug = format!("{:?}", id);
        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains("42"));
    }

    #[test]
    fn tenant_id_roundtrip() {
        let id = TenantId::new(100);
        assert_eq!(id.get(), 100);
    }

    #[test]
    fn email_address_id_roundtrip() {
        let id = EmailAddressId::new(200);
        assert_eq!(id.get(), 200);
    }

    #[test]
    fn ids_are_comparable() {
        let id1 = UserId::new(1);
        let id2 = UserId::new(2);
        assert!(id1 < id2);
        assert!(id1 != id2);
    }
}
