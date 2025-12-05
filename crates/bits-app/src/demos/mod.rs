/// List of demo subdomains (for reservation)
pub const SUBDOMAINS: &[&str] = &["jcf", "emporium", "charlie"];

#[cfg(feature = "server")]
pub mod charlie;
#[cfg(feature = "server")]
pub mod emporium;
#[cfg(feature = "server")]
pub mod jcf;
#[cfg(feature = "server")]
mod template;

#[cfg(feature = "server")]
use crate::tenant::Handle;
#[cfg(feature = "server")]
use dioxus::prelude::*;

/// Get demo component by handle
#[cfg(feature = "server")]
pub fn get_demo(handle: &Handle) -> Option<fn() -> Element> {
    match handle.as_str() {
        "jcf" => Some(jcf::Component),
        "emporium" => Some(emporium::Component),
        "charlie" => Some(charlie::Component),
        _ => None,
    }
}
