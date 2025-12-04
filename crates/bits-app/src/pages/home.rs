use dioxus::prelude::*;

/// Home page
#[component]
pub fn Home() -> Element {
    let realm = use_context::<Resource<Result<crate::Realm>>>();
    let t = crate::i18n::use_translation();

    rsx! {
        div { class: "flex min-h-full items-center justify-center p-8",
            h1 { class: "text-4xl font-bold text-gray-900 dark:text-gray-100",
                match realm() {
                    Some(Ok(crate::Realm::Tenancy(tenant))) => rsx! { "{tenant.name}" },
                    Some(Ok(crate::Realm::Platform)) => rsx! { "{t.t(\"home-welcome\")}" },
                    Some(Ok(crate::Realm::UnknownTenant)) => rsx! { "{t.t(\"home-unknown-tenant\")}" },
                    Some(Err(_)) => rsx! { "{t.t(\"home-welcome\")}" },
                    None => rsx! { "{t.t(\"common-loading\")}" },
                }
            }
        }
    }
}
