#![forbid(unsafe_code)]
#![deny(clippy::fallible_impl_from)]
#![deny(clippy::fn_params_excessive_bools)]
#![deny(clippy::indexing_slicing)]
#![deny(clippy::must_use_candidate)]
#![deny(clippy::unneeded_field_pattern)]
#![deny(clippy::wildcard_enum_match_arm)]

pub mod assertions;
pub mod client;
pub mod fixtures;
pub mod request;
pub mod seeds;
pub mod server;

pub use fixtures::config;
