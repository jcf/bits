use super::wrapper::Wrapper;
use crate::html_tag::HtmlTag;
use dioxus::prelude::*;

#[component]
pub fn Card(
    #[props(default = HtmlTag::Div)] using: HtmlTag,
    #[props(default = "".to_string())] class: String,
    children: Element,
) -> Element {
    let merged_class = format!(
        "overflow-hidden bg-white shadow-sm sm:rounded-lg dark:bg-neutral-800/50 dark:shadow-none dark:outline dark:-outline-offset-1 dark:outline-white/10 {}",
        class
    );

    rsx! {
        Wrapper {
            using,
            class: merged_class,
            div {
                class: "px-4 py-5 sm:p-6",
                {children}
            }
        }
    }
}
