use super::template::{Layout, Link, Profile};
use crate::components::{EmailIcon, GitHubIcon, MastodonIcon};
use dioxus::prelude::*;

#[component]
pub fn Component() -> Element {
    let profile = Profile {
        display_name: "James Conroy-Finn".to_string(),
        bio: "Building Bits. Your audience. Your revenue. Your rules.".to_string(),
        avatar_url: "https://images.bits.page/jcf/avatar.jpg".to_string(),
        banner_url: "https://images.bits.page/jcf/banner.jpg".to_string(),
        links: vec![
            Link {
                title: "GitHub".to_string(),
                url: "https://github.com/jcf".to_string(),
                icon: rsx! { GitHubIcon { class: "size-6 text-gray-900 dark:text-white" } },
            },
            Link {
                title: "Mastodon".to_string(),
                url: "https://mastodon.social/@jcf".to_string(),
                icon: rsx! { MastodonIcon { class: "size-6 text-gray-900 dark:text-white" } },
            },
            Link {
                title: "Email".to_string(),
                url: "mailto:james@invetica.co.uk".to_string(),
                icon: rsx! { EmailIcon { class: "size-6 text-gray-900 dark:text-white" } },
            },
        ],
    };

    rsx! { Layout { profile } }
}
