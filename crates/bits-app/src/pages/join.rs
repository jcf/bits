use dioxus::prelude::*;

#[component]
pub fn Join() -> Element {
    use crate::auth::{join, JoinForm};
    use crate::components::Input;

    let mut join_action = use_action(join);
    let nav = navigator();
    let t = crate::i18n::use_translation();

    rsx! {
        div { class: "flex min-h-full items-center justify-center px-4 py-12",
            div { class: "w-full max-w-sm space-y-10",
                h2 { class: "mt-10 text-center text-2xl font-bold text-gray-900 dark:text-white",
                    "{t.t(\"auth-create-account-title\")}"
                }
                if let Some(Err(err)) = join_action.value() {
                    div { class: "text-red-500 text-sm text-center", "{err}" }
                }
                if let Some(Ok(_)) = join_action.value() {
                    div { class: "text-green-500 text-sm text-center",
                        "{t.t(\"auth-account-created-success\")}"
                    }
                }
                form {
                    method: "post",
                    class: "space-y-6",
                    onsubmit: move |evt: FormEvent| async move {
                        evt.prevent_default();
                        let form: JoinForm = evt.parsed_values().unwrap();
                        join_action.call(dioxus::fullstack::Form(form)).await;
                        if join_action.value().and_then(|r| r.ok()).is_some() {
                            nav.push(crate::app::Route::VerifyEmail {});
                        }
                    },
                    div {
                        Input {
                            id: "email",
                            input_type: "email",
                            name: "email",
                            placeholder: t.t("form-email-placeholder"),
                        }
                    }
                    div {
                        Input {
                            id: "password",
                            input_type: "password",
                            name: "password",
                            placeholder: t.t("form-password-placeholder"),
                        }
                    }
                    crate::components::Button {
                        button_type: "submit",
                        variant: crate::components::ButtonVariant::Primary,
                        size: crate::components::ButtonSize::LG,
                        loading: join_action.pending(),
                        class: "w-full",
                        if join_action.pending() {
                            "{t.t(\"auth-create-account-loading\")}"
                        } else {
                            "{t.t(\"auth-create-account-button\")}"
                        }
                    }
                }
                p { class: "text-center text-sm text-gray-500",
                    "{t.t(\"auth-already-member\")} "
                    Link {
                        to: crate::app::Route::Auth {},
                        class: "text-indigo-600 hover:text-indigo-500",
                        "{t.t(\"auth-sign-in-button\")}"
                    }
                }
            }
        }
    }
}
