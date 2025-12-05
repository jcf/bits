use crate::i18n::use_translation;
use crate::subdomain::{check_subdomain, SubdomainStatus};
use dioxus::prelude::*;

#[component]
pub fn Landing() -> Element {
    let t = use_translation();
    let realm = use_context::<Resource<Result<crate::Realm>>>();
    let mut subdomain_input = use_signal(String::new);
    let mut status = use_signal(|| Option::<SubdomainStatus>::None);
    let mut show_modal = use_signal(|| false);
    let mut checking = use_signal(|| false);

    let platform_domain = match realm() {
        Some(Ok(crate::Realm::Platform { domain })) => domain,
        _ => "bits.page".to_string(),
    };

    // Debounced check
    use_effect(move || {
        let input = subdomain_input();
        if input.len() >= 3 {
            checking.set(true);
            spawn(async move {
                match check_subdomain(input).await {
                    Ok(s) => status.set(Some(s)),
                    Err(_) => status.set(None), // Clear status on error
                }
                checking.set(false);
            });
        } else {
            status.set(None);
        }
    });

    rsx! {
        div { class: "flex mt-20 flex-col items-center justify-center p-8",
            div { class: "max-w-2xl w-full space-y-8 text-center",
                // Hero
                h1 {
                    class: "text-5xl font-bold text-gray-900 dark:text-white mb-4",
                    { t.t("landing-tagline") }
                }
                p {
                    class: "text-xl text-gray-600 dark:text-gray-400 mb-8",
                    { t.t("landing-description") }
                }

                // Subdomain checker
                div { class: "bg-white dark:bg-gray-800 p-8 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700",
                    h2 {
                        class: "text-2xl font-semibold text-gray-900 dark:text-white mb-6",
                        { t.t("landing-reserve-username") }
                    }

                    div { class: "mb-4",
                        div { class: "flex items-center gap-2",
                            div { class: "flex-1 grid grid-cols-1",
                                input {
                                    r#type: "text",
                                    placeholder: t.t("subdomain-input-placeholder"),
                                    autocomplete: "off",
                                    "data-1p-ignore": "true",
                                    class: "col-start-1 row-start-1 w-full px-4 py-3 pr-10 border border-gray-300 dark:border-gray-600 rounded-lg text-lg focus:outline-none focus:ring-2 focus:ring-indigo-500 dark:bg-gray-700 dark:text-white",
                                    value: "{subdomain_input()}",
                                    oninput: move |evt| subdomain_input.set(evt.value()),
                                }
                                if checking() {
                                    svg {
                                        view_box: "0 0 20 20",
                                        fill: "none",
                                        class: "pointer-events-none col-start-1 row-start-1 mr-3 size-5 self-center justify-self-end text-gray-400 animate-spin",
                                        circle {
                                            cx: "10",
                                            cy: "10",
                                            r: "8",
                                            stroke: "currentColor",
                                            "stroke-width": "2",
                                            "stroke-dasharray": "50",
                                            "stroke-dashoffset": "25",
                                        }
                                    }
                                }
                            }
                            span { class: "text-gray-600 dark:text-gray-400 text-lg", ".bits.page" }
                        }
                    }

                    // Status indicator
                    div { class: "mt-4",
                        if let Some(ref s) = status() {
                            {
                                let msg = t.t(s.translation_key());
                                if s.is_available() {
                                    rsx! {
                                        div { class: "alert-animate rounded-md bg-green-50 p-4",
                                            div { class: "flex",
                                                div { class: "shrink-0",
                                                    svg {
                                                        view_box: "0 0 20 20",
                                                        fill: "currentColor",
                                                        "aria-hidden": "true",
                                                        class: "size-5 text-green-400",
                                                        path {
                                                            d: "M10 18a8 8 0 1 0 0-16 8 8 0 0 0 0 16Zm3.857-9.809a.75.75 0 0 0-1.214-.882l-3.483 4.79-1.88-1.88a.75.75 0 1 0-1.06 1.061l2.5 2.5a.75.75 0 0 0 1.137-.089l4-5.5Z",
                                                            "clip-rule": "evenodd",
                                                            "fill-rule": "evenodd",
                                                        }
                                                    }
                                                }
                                                div { class: "ml-3",
                                                    p { class: "text-sm font-medium text-green-800", "{msg}" }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    rsx! {
                                        div { class: "alert-animate border-l-4 border-yellow-400 bg-yellow-50 p-4",
                                            div { class: "flex",
                                                div { class: "shrink-0",
                                                    svg {
                                                        view_box: "0 0 20 20",
                                                        fill: "currentColor",
                                                        "aria-hidden": "true",
                                                        class: "size-5 text-yellow-400",
                                                        path {
                                                            d: "M8.485 2.495c.673-1.167 2.357-1.167 3.03 0l6.28 10.875c.673 1.167-.17 2.625-1.516 2.625H3.72c-1.347 0-2.189-1.458-1.515-2.625L8.485 2.495ZM10 5a.75.75 0 0 1 .75.75v3.5a.75.75 0 0 1-1.5 0v-3.5A.75.75 0 0 1 10 5Zm0 9a1 1 0 1 0 0-2 1 1 0 0 0 0 2Z",
                                                            "clip-rule": "evenodd",
                                                            "fill-rule": "evenodd",
                                                        }
                                                    }
                                                }
                                                div { class: "ml-3",
                                                    p { class: "text-sm text-yellow-700", "{msg}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Reserve button
                    button {
                        class: "w-full mt-4 px-6 py-3 bg-indigo-600 hover:bg-indigo-700 text-white font-semibold rounded-lg transition disabled:opacity-50 disabled:cursor-not-allowed",
                        disabled: !status().as_ref().map_or(false, |s| s.is_available()),
                        onclick: move |_| show_modal.set(true),
                        "Reserve"
                    }
                }

                // Features
                div { class: "mt-12 grid grid-cols-1 md:grid-cols-3 gap-6 text-left",
                    div { class: "p-6 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700",
                        h3 { class: "font-semibold text-gray-900 dark:text-white mb-2", "Your Data" }
                        p { class: "text-gray-600 dark:text-gray-400", { t.t("landing-feature-self-host") } }
                    }
                    div { class: "p-6 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700",
                        h3 { class: "font-semibold text-gray-900 dark:text-white mb-2", "Your Revenue" }
                        p { class: "text-gray-600 dark:text-gray-400", { t.t("landing-feature-direct-payments") } }
                    }
                    div { class: "p-6 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700",
                        h3 { class: "font-semibold text-gray-900 dark:text-white mb-2", "Your Rules" }
                        p { class: "text-gray-600 dark:text-gray-400", { t.t("landing-feature-permissive") } }
                    }
                }

                // Demo examples
                div { class: "mt-12 text-center",
                    h2 { class: "text-2xl font-semibold text-gray-900 dark:text-white mb-6", "Examples" }
                    div { class: "flex flex-wrap justify-center gap-4",
                        for handle in crate::demos::SUBDOMAINS {
                            a {
                                href: "https://{handle}.{platform_domain}/",
                                class: "px-6 py-3 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg hover:border-indigo-500 dark:hover:border-indigo-400 transition font-medium text-gray-900 dark:text-white",
                                "{handle}.bits.page"
                            }
                        }
                    }
                }
            }
        }

        // Modal
        if show_modal() {
            div {
                class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50",
                onclick: move |_| show_modal.set(false),
                div {
                    class: "bg-white dark:bg-gray-800 rounded-lg p-8 max-w-md w-full",
                    onclick: move |evt| evt.stop_propagation(),

                    h3 {
                        class: "text-2xl font-bold text-gray-900 dark:text-white mb-4",
                        { t.t("landing-modal-thanks") }
                    }
                    p {
                        class: "text-gray-600 dark:text-gray-400 mb-6",
                        { t.t("landing-modal-description") }
                        " "
                        strong { "{subdomain_input()}.bits.page" }
                        ", please get in touch:"
                    }
                    a {
                        href: "mailto:hello@bits.page?subject=Reserve%20{subdomain_input()}&body=I'd%20like%20to%20reserve%20{subdomain_input()}.bits.page",
                        class: "block w-full px-6 py-3 bg-indigo-600 hover:bg-indigo-700 text-white font-semibold rounded-lg text-center transition",
                        { t.t("landing-modal-email") }
                    }
                    button {
                        class: "mt-4 w-full px-6 py-3 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 font-semibold rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition",
                        onclick: move |_| show_modal.set(false),
                        "Close"
                    }
                }
            }
        }
    }
}
