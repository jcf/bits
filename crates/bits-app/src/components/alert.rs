use dioxus::prelude::*;

use super::{CheckCircleIcon, ExclamationTriangleIcon, XCircleIcon};

#[derive(Clone, Copy, PartialEq)]
pub enum AlertVariant {
    Success,
    Notice,
    Error,
}

#[component]
pub fn Alert(variant: AlertVariant, message: String) -> Element {
    let (container_class, text_class, icon) = match variant {
        AlertVariant::Success => (
            "alert-animate rounded-md bg-green-50 p-4",
            "text-sm font-medium text-green-800",
            rsx! { CheckCircleIcon { class: "text-green-400".to_string() } },
        ),
        AlertVariant::Notice => (
            "alert-animate border-l-4 border-yellow-400 bg-yellow-50 p-4",
            "text-sm text-yellow-700",
            rsx! { ExclamationTriangleIcon { class: "text-yellow-400".to_string() } },
        ),
        AlertVariant::Error => (
            "alert-animate rounded-md bg-red-50 p-4",
            "text-sm font-medium text-red-800",
            rsx! { XCircleIcon { class: "text-red-400".to_string() } },
        ),
    };

    rsx! {
        div { class: container_class,
            div { class: "flex",
                div { class: "shrink-0", {icon} }
                div { class: "ml-3",
                    p { class: text_class, "{message}" }
                }
            }
        }
    }
}
