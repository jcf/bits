use super::template::{Layout, Link, Profile};
use crate::components::{EmailIcon, InstagramIcon, ShoppingBagIcon};
use dioxus::prelude::*;

#[component]
pub fn Component() -> Element {
    let profile = Profile {
        display_name: "Jimmy's Leather Emporium".to_string(),
        bio: "Handcrafted leather goods for discerning customers. Custom orders welcome."
            .to_string(),
        avatar_url: "https://images.bits.page/emporium/avatar.jpg".to_string(),
        banner_url: "https://images.bits.page/emporium/workshop.jpg".to_string(),
        links: vec![
            Link {
                title: "Shop Catalog".to_string(),
                url: "#".to_string(),
                icon: rsx! { ShoppingBagIcon { class: "size-6 text-gray-900 dark:text-white" } },
            },
            Link {
                title: "Custom Orders".to_string(),
                url: "mailto:orders@example.com".to_string(),
                icon: rsx! { EmailIcon { class: "size-6 text-gray-900 dark:text-white" } },
            },
            Link {
                title: "Instagram".to_string(),
                url: "#".to_string(),
                icon: rsx! { InstagramIcon { class: "size-6 text-gray-900 dark:text-white" } },
            },
        ],
    };

    rsx! { Layout { profile } }
}
