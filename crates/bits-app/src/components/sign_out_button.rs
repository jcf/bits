use dioxus::prelude::*;

#[component]
pub fn SignOutButton() -> Element {
    use crate::auth::{sign_out, SessionState};

    let mut session = use_context::<Resource<Result<SessionState>>>();
    let mut sign_out_action = use_action(sign_out);
    let nav = navigator();
    let t = crate::i18n::use_translation();

    use_effect(move || {
        if sign_out_action.value().and_then(|r| r.ok()).is_some() {
            session.set(Some(Ok(SessionState::Anonymous)));
            nav.push(crate::app::Route::Home {});
        }
    });

    rsx! {
        crate::components::Button {
            variant: crate::components::ButtonVariant::Secondary,
            size: crate::components::ButtonSize::SM,
            loading: sign_out_action.pending(),
            onclick: move |_| sign_out_action.call(),
            if sign_out_action.pending() {
                "{t.t(\"auth-sign-out-loading\")}"
            } else {
                "{t.t(\"auth-sign-out-button\")}"
            }
        }
    }
}
