#![forbid(unsafe_code)]
#![deny(clippy::fallible_impl_from)]
#![deny(clippy::fn_params_excessive_bools)]
#![deny(clippy::indexing_slicing)]
#![deny(clippy::must_use_candidate)]
#![deny(clippy::unneeded_field_pattern)]
#![deny(clippy::wildcard_enum_match_arm)]

pub mod email;
pub mod id;
pub mod password;
pub mod password_hash;

// Re-exports for convenience
pub use email::{Email, EmailError};
pub use id::{EmailAddressId, TenantId, UserId};
pub use password::Password;
pub use password_hash::PasswordHash;
