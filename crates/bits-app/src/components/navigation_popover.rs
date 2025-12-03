use dioxus::prelude::*;

#[component]
pub fn NavigationPopover(
    label: String,
    is_open: Signal<bool>,
    switching: Signal<bool>,
    on_toggle: EventHandler<()>,
    children: Element,
) -> Element {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::prelude::*;
        use wasm_bindgen::JsCast;

        use_effect(move || {
            if switching() {
                let window = web_sys::window().expect("no window");
                let mut switching = switching;
                let closure = Closure::wrap(Box::new(move || {
                    switching.set(false);
                }) as Box<dyn FnMut()>);
                let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    50,
                );
                closure.forget();
            }
        });
    }

    rsx! {
        div { class: "flex", "data-popover": "{label}",
            div { class: "relative flex",
                button {
                    r#type: "button",
                    class: if is_open() {
                        "group relative flex items-center justify-center text-sm font-semibold transition-colors duration-200 ease-out text-indigo-600 dark:text-indigo-400"
                    } else {
                        "group relative flex items-center justify-center text-sm font-semibold text-gray-700 transition-colors duration-200 ease-out hover:text-gray-900 dark:text-gray-300 dark:hover:text-white"
                    },
                    onclick: move |evt| {
                        evt.stop_propagation();
                        on_toggle.call(());
                    },
                    "{label}"
                    span {
                        "aria-hidden": "true",
                        class: if is_open() {
                            "absolute inset-x-0 -bottom-px z-30 h-0.5 transition duration-200 ease-out bg-indigo-600 dark:bg-indigo-400"
                        } else {
                            "absolute inset-x-0 -bottom-px z-30 h-0.5 transition duration-200 ease-out"
                        },
                    }
                }
            }
            div {
                class: if switching() {
                    if is_open() {
                        "absolute inset-x-0 top-full z-20 w-full bg-white text-sm text-gray-500 opacity-100 translate-y-0 dark:bg-gray-900 dark:text-gray-400"
                    } else {
                        "absolute inset-x-0 top-full z-20 w-full bg-white text-sm text-gray-500 opacity-0 translate-y-0 pointer-events-none dark:bg-gray-900 dark:text-gray-400"
                    }
                } else {
                    if is_open() {
                        "absolute inset-x-0 top-full z-20 w-full bg-white text-sm text-gray-500 transition-all duration-200 ease-out opacity-100 translate-y-0 dark:bg-gray-900 dark:text-gray-400"
                    } else {
                        "absolute inset-x-0 top-full z-20 w-full bg-white text-sm text-gray-500 transition-all duration-150 ease-in opacity-0 translate-y-1 pointer-events-none dark:bg-gray-900 dark:text-gray-400"
                    }
                },
                div { "aria-hidden": "true", class: "absolute inset-0 top-1/2 bg-white shadow-sm dark:bg-gray-900" }
                div { class: "relative bg-white dark:bg-gray-900",
                    div { class: "mx-auto max-w-7xl px-8",
                        {children}
                    }
                }
            }
        }
    }
}
