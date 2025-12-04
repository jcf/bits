pub mod charlie;
pub mod emporium;
pub mod jcf;
mod template;

use crate::tenant::Handle;
use dioxus::prelude::*;

/// Get demo component by handle
pub fn get_demo(handle: &Handle) -> Option<fn() -> Element> {
    match handle.as_str() {
        "jcf" => Some(jcf::Component),
        "emporium" => Some(emporium::Component),
        "charlie" => Some(charlie::Component),
        _ => None,
    }
}

/// List of demo subdomains (for reservation)
pub const SUBDOMAINS: &[&str] = &["jcf", "emporium", "charlie"];
