use crate::i18n::use_translation;
use dioxus::prelude::*;
use fluent_bundle::FluentArgs;

#[component]
pub fn VerifyEmail() -> Element {
    use crate::auth::{
        resend_verification_code, verify_email_code, ResendForm, SessionState, VerifyEmailForm,
    };

    let t = use_translation();
    let session = use_context::<Resource<Result<SessionState>>>();
    let mut code = use_signal(String::new);
    let mut verify_action = use_action(verify_email_code);
    let mut resend_action = use_action(resend_verification_code);
    let nav = navigator();

    // Get email from authenticated session and store in signal
    let email = use_signal(|| match session() {
        Some(Ok(SessionState::Authenticated(user))) => user.email.clone(),
        _ => String::new(),
    });

    // Redirect if not authenticated
    match session() {
        Some(Ok(SessionState::Authenticated(_))) => {
            // Continue
        }
        Some(Ok(SessionState::Anonymous)) => {
            nav.push(crate::app::Route::Auth {});
            return rsx! { div { "Redirecting..." } };
        }
        Some(Err(_)) | None => {
            return rsx! { div { "Loading..." } };
        }
    }

    // Calculate cooldown from resend response
    let mut cooldown_secs = use_signal(|| 0u32);

    // Countdown timer effect
    #[cfg(target_arch = "wasm32")]
    use_effect(move || {
        spawn(async move {
            while cooldown_secs() > 0 {
                gloo_timers::future::TimeoutFuture::new(1000).await;
                if cooldown_secs() > 0 {
                    cooldown_secs.set(cooldown_secs() - 1);
                }
            }
        });
    });

    // Update cooldown when resend succeeds
    use_effect(move || {
        if let Some(Ok(_response)) = resend_action.value() {
            #[cfg(target_arch = "wasm32")]
            {
                let now = (js_sys::Date::now() / 1000.0) as i64;
                let remaining = (_response().next_resend_at - now).max(0) as u32;
                cooldown_secs.set(remaining);
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                // Server-side - shouldn't happen but handle gracefully
                cooldown_secs.set(60);
            }
        }
    });

    // Navigate to home on successful verification
    use_effect(move || {
        if verify_action.value().and_then(|r| r.ok()).is_some() {
            nav.push(crate::app::Route::Auth {});
        }
    });

    rsx! {
        div { class: "flex min-h-full items-center justify-center px-4 py-12",
            div { class: "w-full max-w-md space-y-6 text-center",
                h2 { class: "text-2xl font-bold text-gray-900 dark:text-white",
                    { t.t("verify-email-title") }
                }
                p { class: "text-sm text-gray-600 dark:text-gray-400",
                    { t.t("verify-email-description") }
                    " "
                    span { class: "font-medium", "{email()}" }
                }

                if let Some(Err(err)) = verify_action.value() {
                    div { class: "rounded-md bg-red-50 dark:bg-red-900/20 p-4",
                        p { class: "text-sm text-red-800 dark:text-red-200", "{err}" }
                    }
                }

                if let Some(Ok(_)) = verify_action.value() {
                    div { class: "rounded-md bg-green-50 dark:bg-green-900/20 p-4",
                        p { class: "text-sm text-green-800 dark:text-green-200",
                            { t.t("verify-email-success") }
                        }
                    }
                }

                form {
                    onsubmit: move |evt: FormEvent| {
                        evt.prevent_default();
                    },
                    div { class: "flex justify-center",
                        input {
                            id: "code",
                            r#type: "text",
                            name: "code",
                            placeholder: "000000",
                            class: "text-center text-3xl tracking-widest font-mono w-64 block rounded-md px-3 py-2 border border-gray-300 text-gray-900 placeholder:text-gray-400 focus:outline-2 focus:-outline-offset-1 focus:outline-indigo-600 dark:border-gray-700 dark:bg-gray-900 dark:text-white dark:placeholder:text-gray-500 dark:focus:outline-indigo-500",
                            autocomplete: "one-time-code",
                            inputmode: "numeric",
                            pattern: r"\d{6}",
                            maxlength: "6",
                            oninput: move |evt: FormEvent| {
                                let value = evt.value()
                                    .chars()
                                    .filter(|c| c.is_numeric())
                                    .take(6)
                                    .collect::<String>();

                                code.set(value.clone());

                                if value.len() == 6 {
                                    verify_action.call(dioxus::fullstack::Form(VerifyEmailForm {
                                        email: email(),
                                        code: value,
                                    }));
                                }
                            }
                        }
                    }
                }

                div { class: "space-y-2",
                    if cooldown_secs() > 0 {
                        {
                            let mut args = FluentArgs::new();
                            args.set("seconds", cooldown_secs());
                            let msg = t.t_with_args("verify-email-resend-cooldown", args);
                            rsx! {
                                p { class: "text-sm text-gray-500", "{msg}" }
                            }
                        }
                    } else {
                        crate::components::Button {
                            variant: crate::components::ButtonVariant::Secondary,
                            size: crate::components::ButtonSize::SM,
                            loading: resend_action.pending(),
                            onclick: move |_| {
                                resend_action.call(dioxus::fullstack::Form(ResendForm {
                                    email: email(),
                                }));
                            },
                            "Resend Code"
                        }
                    }

                    if let Some(Err(err)) = resend_action.value() {
                        p { class: "text-sm text-red-600 dark:text-red-400", "{err}" }
                    }
                }

                p { class: "text-sm text-gray-500",
                    Link {
                        to: crate::app::Route::Auth {},
                        class: "text-indigo-600 hover:text-indigo-500",
                        "Back to sign in"
                    }
                }
            }
        }
    }
}
