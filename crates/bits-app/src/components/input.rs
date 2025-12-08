use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct InputProps {
    pub id: String,
    pub input_type: String,
    pub name: String,
    pub placeholder: String,
    #[props(default)]
    pub value: Option<String>,
    #[props(default)]
    pub oninput: Option<EventHandler<FormEvent>>,
}

#[component]
pub fn Input(props: InputProps) -> Element {
    let autocapitalize = if props.input_type == "email" {
        Some("none")
    } else {
        None
    };

    rsx! {
        input {
            id: "{props.id}",
            r#type: "{props.input_type}",
            name: "{props.name}",
            required: true,
            autocomplete: "off",
            autocapitalize,
            placeholder: "{props.placeholder}",
            class: "block w-full rounded-md px-3 py-2 border border-gray-300 text-gray-900 placeholder:text-gray-400 focus:outline-2 focus:-outline-offset-1 focus:outline-indigo-600 dark:border-gray-700 dark:bg-gray-900 dark:text-white dark:placeholder:text-gray-500 dark:focus:outline-indigo-500",
            value: props.value.unwrap_or_default(),
            oninput: move |evt| {
                if let Some(handler) = &props.oninput {
                    handler.call(evt);
                }
            },
        }
    }
}
