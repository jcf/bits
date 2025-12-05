use crate::pages::Landing;
use dioxus::prelude::*;

/// Home page
#[component]
pub fn Home() -> Element {
    let realm = use_context::<Resource<Result<crate::Realm>>>();
    let t = crate::i18n::use_translation();

    match realm() {
        Some(Ok(crate::Realm::Demo(handle))) => {
            if let Some(demo_component) = crate::demos::get_demo(&handle) {
                rsx! { {demo_component()} }
            } else {
                rsx! { NotFound {} }
            }
        }
        Some(Ok(crate::Realm::Creator(tenant))) => {
            // Future: Real tenant profile from database
            rsx! {
                div { class: "flex min-h-full items-center justify-center p-8",
                    h1 { class: "text-4xl font-bold text-gray-900 dark:text-gray-100",
                        "{tenant.name}"
                    }
                }
            }
        }
        Some(Ok(crate::Realm::Platform { .. })) => {
            rsx! { Landing {} }
        }
        Some(Ok(crate::Realm::NotFound)) => rsx! { NotFound {} },
        Some(Err(_)) => rsx! {
            div { class: "flex min-h-full items-center justify-center p-8",
                h1 { class: "text-4xl font-bold text-gray-900 dark:text-gray-100",
                    "{t.t(\"common-error\")}"
                }
            }
        },
        None => rsx! {
            div { class: "flex min-h-full items-center justify-center p-8",
                h1 { class: "text-4xl font-bold text-gray-900 dark:text-gray-100",
                    "{t.t(\"common-loading\")}"
                }
            }
        },
    }
}

#[component]
fn NotFound() -> Element {
    let t = crate::i18n::use_translation();

    rsx! {
        div { class: "flex min-h-full items-center justify-center p-8",
            div { class: "text-center",
                h1 { class: "text-4xl font-bold text-red-500",
                    "{t.t(\"error-404-title\")}"
                }
                p { class: "mt-4 text-gray-600 dark:text-gray-400",
                    "{t.t(\"error-404-message\")}"
                }
            }
        }
    }
}
