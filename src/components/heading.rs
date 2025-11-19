use super::wrapper::Wrapper;
use crate::html_tag::HtmlTag;
use dioxus::prelude::*;

#[component]
pub fn Heading(
    #[props(default = HtmlTag::H1)] using: HtmlTag,
    #[props(default = "".to_string())] class: String,
    children: Element,
) -> Element {
    let merged_class = format!(
        "font-3xl font-bold text-neutral-900 dark:text-neutral-100 {}",
        class
    );

    rsx! {
        Wrapper {
            using,
            class: merged_class,
            {children}
        }
    }
}
