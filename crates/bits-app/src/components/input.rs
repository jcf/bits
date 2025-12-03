use dioxus::prelude::*;

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
            autocapitalize,
            placeholder: "{placeholder}",
            class: "block w-full rounded-md px-3 py-2 border border-gray-300 text-gray-900 placeholder:text-gray-400 focus:outline-2 focus:-outline-offset-1 focus:outline-indigo-600 dark:border-gray-700 dark:bg-gray-900 dark:text-white dark:placeholder:text-gray-500 dark:focus:outline-indigo-500",
        }
    }
}
