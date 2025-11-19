use super::wrapper::Wrapper;
use crate::html_tag::HtmlTag;
use dioxus::prelude::*;

#[component]
pub fn Container(
    #[props(default = HtmlTag::Div)] using: HtmlTag,
    #[props(default = "".to_string())] class: String,
    children: Element,
) -> Element {
    let merged_class = format!("mx-auto max-w-7xl sm:px-6 lg:px-8 {}", class);

    rsx! {
        Wrapper {
            using,
            class: merged_class,
            {children}
        }
    }
}
