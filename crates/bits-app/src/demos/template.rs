use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct Profile {
    pub display_name: String,
    pub bio: String,
    pub avatar_url: String,
    pub banner_url: String,
    pub links: Vec<Link>,
}

#[derive(Clone, PartialEq)]
pub struct Link {
    pub title: String,
    pub url: String,
    pub icon: Element,
}

#[component]
pub fn Layout(profile: Profile) -> Element {
    rsx! {
        // Demo banner
        div {
            class: "bg-yellow-50 border-b border-yellow-200 p-3 text-center",
            p {
                class: "text-sm text-yellow-800",
                "🎭 Demo Profile - Showcasing platform features"
            }
        }

        div { class: "max-w-2xl mx-auto",
            // Banner
            img {
                class: "w-full h-48 object-cover",
                src: "{profile.banner_url}"
            }

            // Avatar + Bio
            div { class: "p-6 text-center",
                img {
                    class: "w-24 h-24 rounded-full mx-auto -mt-12 border-4 border-white",
                    src: "{profile.avatar_url}"
                }
                h1 {
                    class: "text-2xl font-bold mt-4 text-gray-900 dark:text-white",
                    "{profile.display_name}"
                }
                p {
                    class: "text-gray-600 dark:text-gray-400 mt-2",
                    "{profile.bio}"
                }
            }

            // Links
            div { class: "space-y-3 p-6",
                for link in profile.links {
                    a {
                        href: "{link.url}",
                        target: "_blank",
                        class: "block p-4 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg hover:shadow-md transition",
                        div { class: "flex items-center gap-3",
                            {link.icon}
                            span { class: "font-medium text-gray-900 dark:text-white", "{link.title}" }
                        }
                    }
                }
            }
        }
    }
}
