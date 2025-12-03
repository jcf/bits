use dioxus::prelude::*;

use crate::auth::{sign_out, User};
use crate::Route;

#[component]
pub fn SignOutLink(class: Option<String>) -> Element {
    let mut session = use_context::<Resource<Result<Option<User>>>>();
    let mut sign_out_action = use_action(sign_out);
    let nav = navigator();
    let t = crate::i18n::use_translation();

    use_effect(move || {
        if sign_out_action.value().and_then(|r| r.ok()).is_some() {
            session.restart();
            nav.push(Route::Home {});
        }
    });

    rsx! {
        button {
            r#type: "button",
            class: class.unwrap_or_default(),
            onclick: move |_| sign_out_action.call(),
            "{t.t(\"auth-sign-out-button\")}"
        }
    }
}
