use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct AuthFormState {
    pub email: Signal<String>,
    pub password: Signal<String>,
}

impl AuthFormState {
    pub fn clear(&mut self) {
        self.email.set(String::new());
        self.password.set(String::new());
    }
}

#[component]
pub fn AuthFormFields() -> Element {
    let mut form_state = use_context::<AuthFormState>();
    let t = crate::i18n::use_translation();

    rsx! {
        div {
            crate::components::Input {
                id: "email".to_string(),
                input_type: "email".to_string(),
                name: "email".to_string(),
                placeholder: t.t("form-email-placeholder"),
                value: Some((form_state.email)()),
                oninput: move |evt: FormEvent| {
                    form_state.email.set(evt.value().clone());
                },
            }
        }
        div {
            crate::components::Input {
                id: "password".to_string(),
                input_type: "password".to_string(),
                name: "password".to_string(),
                placeholder: t.t("form-password-placeholder"),
                value: Some((form_state.password)()),
                oninput: move |evt: FormEvent| {
                    form_state.password.set(evt.value().clone());
                },
            }
        }
    }
}
