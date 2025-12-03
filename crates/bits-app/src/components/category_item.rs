use dioxus::prelude::*;

#[component]
pub fn CategoryItem(name: String) -> Element {
    let t = crate::i18n::use_translation();

    rsx! {
        div { class: "group relative",
            div { class: "aspect-square w-full rounded-md bg-gray-200 dark:bg-gray-700" }
            div { class: "mt-4 block font-medium text-gray-900 dark:text-white",
                span { "aria-hidden": "true", class: "absolute inset-0 z-10" }
                "{name}"
            }
            p { "aria-hidden": "true", class: "mt-1 text-gray-600 dark:text-gray-400", "{t.t(\"category-shop-now\")}" }
        }
    }
}

#[component]
pub fn MobileCategoryItem(name: String) -> Element {
    let t = crate::i18n::use_translation();

    rsx! {
        div { class: "group relative",
            div { class: "aspect-square w-full rounded-md bg-gray-200 dark:bg-gray-700" }
            div { class: "mt-6 block text-sm font-medium text-gray-900 dark:text-white",
                span { "aria-hidden": "true", class: "absolute inset-0 z-10" }
                "{name}"
            }
            p { "aria-hidden": "true", class: "mt-1 text-sm text-gray-500 dark:text-gray-400", "{t.t(\"category-shop-now\")}" }
        }
    }
}
