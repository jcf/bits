use super::template::{Layout, Link, Profile};
use crate::components::{ChatBubbleIcon, ExclamationTriangleIcon, PlayIcon};
use dioxus::prelude::*;

#[component]
pub fn Component() -> Element {
    let profile = Profile {
        display_name: "Charlie's Countryside Adventures".to_string(),
        bio: "Adult content creator. Subscribe for exclusive content. 18+ only.".to_string(),
        avatar_url: "https://images.bits.page/charlie/avatar.jpg".to_string(),
        banner_url: "https://images.bits.page/charlie/countryside.jpg".to_string(),
        links: vec![
            Link {
                title: "Subscribe (18+)".to_string(),
                url: "#".to_string(),
                icon: rsx! {
                    ExclamationTriangleIcon { class: "size-6 text-gray-900 dark:text-white" }
                },
            },
            Link {
                title: "Message Me".to_string(),
                url: "#".to_string(),
                icon: rsx! { ChatBubbleIcon { class: "size-6 text-gray-900 dark:text-white" } },
            },
            Link {
                title: "Video Library".to_string(),
                url: "#".to_string(),
                icon: rsx! { PlayIcon { class: "size-6 text-gray-900 dark:text-white" } },
            },
        ],
    };

    rsx! { Layout { profile } }
}
