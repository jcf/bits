use dioxus::prelude::*;

#[component]
pub fn Auth() -> Element {
    use crate::auth::{auth, AuthForm, SessionState};
    use crate::components::{AuthFormFields, AuthFormState};

    let mut session = use_context::<Resource<Result<SessionState>>>();
    let mut form_state = use_context::<AuthFormState>();
    let mut auth_action = use_action(auth);
    let nav = navigator();
    let t = crate::i18n::use_translation();

    rsx! {
        div { class: "flex min-h-full items-center justify-center px-4 py-12",
            div { class: "w-full max-w-sm space-y-10",
                h2 { class: "mt-10 text-center text-2xl font-bold text-gray-900 dark:text-white",
                    "{t.t(\"auth-sign-in-title\")}"
                }
                if let Some(Err(err)) = auth_action.value() {
                    div { class: "text-red-500 text-sm text-center", "{err}" }
                }
                form {
                    method: "post",
                    class: "space-y-6",
                    onsubmit: move |evt: FormEvent| async move {
                        evt.prevent_default();
                        let form: AuthForm = evt.parsed_values().unwrap();
                        auth_action.call(dioxus::fullstack::Form(form)).await;
                        if let Some(Ok(user)) = auth_action.value() {
                            session.set(Some(Ok(SessionState::Authenticated(user()))));
                            form_state.clear();
                            // Redirect based on verification status
                            if user().verified {
                                nav.push(crate::app::Route::Home {});
                            } else {
                                nav.push(crate::app::Route::VerifyEmail {});
                            }
                        }
                    },
                    AuthFormFields {}
                    crate::components::Button {
                        button_type: "submit",
                        variant: crate::components::ButtonVariant::Primary,
                        size: crate::components::ButtonSize::LG,
                        loading: auth_action.pending(),
                        class: "w-full",
                        if auth_action.pending() {
                            "{t.t(\"auth-sign-in-loading\")}"
                        } else {
                            "{t.t(\"auth-sign-in-button\")}"
                        }
                    }
                }
                p { class: "text-center text-sm text-gray-500",
                    "{t.t(\"auth-not-member\")} "
                    Link {
                        to: crate::app::Route::Join {},
                        class: "text-indigo-600 hover:text-indigo-500",
                        "{t.t(\"auth-create-account-link\")}"
                    }
                }
            }
        }
    }
}
