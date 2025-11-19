use crate::html_tag::HtmlTag;
use dioxus::prelude::*;

#[derive(PartialEq, Clone, Props)]
pub struct WrapperProps {
    #[props(default = HtmlTag::Div)]
    pub using: HtmlTag,
    #[props(default = "".to_string())]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn Wrapper(props: WrapperProps) -> Element {
    let class = props.class;

    match props.using {
        HtmlTag::Article => rsx! { article { class, {props.children} } },
        HtmlTag::Div => rsx! { div { class, {props.children} } },
        HtmlTag::Footer => rsx! { footer { class, {props.children} } },
        HtmlTag::H1 => rsx! { h1 { class, {props.children }}},
        HtmlTag::H2 => rsx! { h2 { class, {props.children }}},
        HtmlTag::H3 => rsx! { h3 { class, {props.children }}},
        HtmlTag::H4 => rsx! { h4 { class, {props.children }}},
        HtmlTag::H5 => rsx! { h5 { class, {props.children }}},
        HtmlTag::H6 => rsx! { h6 { class, {props.children }}},
        HtmlTag::Header => rsx! { header { class, {props.children} } },
        HtmlTag::Main => rsx! { main { class, {props.children} } },
        HtmlTag::Nav => rsx! { nav { class, {props.children} } },
        HtmlTag::Section => rsx! { section { class, {props.children} } },
        _ => panic!("HtmlTag variant {:?} not implemented in Wrapper component", props.using),
    }
}
