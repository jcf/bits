use dioxus::prelude::*;

#[component]
pub fn NotFound() -> Element {
    let t = crate::i18n::use_translation();

    rsx! {
        div { class: "text-red-500",
            h1 { "{t.t(\"error-404-title\")}" }
            p { "{t.t(\"error-404-message\")}" }
        }
    }
}
