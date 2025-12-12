use dioxus::prelude::*;

#[component]
pub fn FeatureCard(title: String, description: String) -> Element {
    rsx! {
        div { class: "p-6 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700",
            h3 { class: "font-semibold text-gray-900 dark:text-white mb-2", "{title}" }
            p { class: "text-gray-600 dark:text-gray-400", "{description}" }
        }
    }
}
