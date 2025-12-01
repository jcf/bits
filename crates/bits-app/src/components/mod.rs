pub mod button;

use dioxus::prelude::*;

pub use button::{Button, ButtonLink, ButtonSize, ButtonVariant, Spinner};

#[component]
pub fn Input(id: String, input_type: String, name: String, placeholder: String) -> Element {
    let autocapitalize = if input_type == "email" {
        Some("none")
    } else {
        None
    };

    rsx! {
        input {
            id: "{id}",
            r#type: "{input_type}",
            name: "{name}",
            required: true,
            autocomplete: "off",
            autocapitalize: autocapitalize,
            placeholder: "{placeholder}",
            class: "block w-full rounded-md px-3 py-2 border border-neutral-300 dark:border-neutral-700 text-neutral-900 dark:text-neutral-100"
        }
    }
}
