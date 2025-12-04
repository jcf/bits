use dioxus::prelude::*;

#[cfg(feature = "server")]
fn get_csrf_token_for_ssr() -> String {
    use dioxus::fullstack::FullstackContext;

    let ctx = match FullstackContext::current() {
        Some(ctx) => ctx,
        None => {
            tracing::warn!("SSR: FullstackContext not available");
            return String::new();
        }
    };

    // Get session to access CSRF token
    let session =
        match ctx.extension::<axum_session::Session<bits_axum_session_sqlx::SessionPgPool>>() {
            Some(s) => s,
            None => {
                tracing::warn!("SSR: Session not found in request extensions");
                return String::new();
            }
        };

    // Try to get existing token from session memory
    match session.get::<String>("csrf_token") {
        Some(token) => {
            tracing::debug!("SSR: Using existing CSRF token from session");
            token
        }
        None => {
            // Generate new token and store in session memory
            let token = crate::csrf::generate_token();
            session.set("csrf_token", &token);
            tracing::debug!("SSR: Generated new CSRF token for session");
            token
        }
    }
}

/// Shared layout component with error handling.
#[component]
pub fn Layout() -> Element {
    use crate::auth::{get_realm, get_session};

    let session = use_server_future(move || async move { get_session().await })?;
    let realm = use_server_future(move || async move { get_realm().await })?;
    let locale = crate::i18n::create_default_locale()
        .unwrap_or_else(|e| panic!("Failed to create default locale: {}", e));

    use_context_provider(|| session);
    use_context_provider(|| realm);
    use_context_provider(|| locale);

    // TODO Make csrf_token a proper type akin to Option<String>.
    #[cfg(feature = "server")]
    let csrf_token = get_csrf_token_for_ssr();
    #[cfg(not(feature = "server"))]
    let csrf_token = String::new();

    rsx! {
        if !csrf_token.is_empty() {
            document::Meta { name: "csrf-token", content: "{csrf_token}" }
        }

        div { class: "flex min-h-screen flex-col",
            header { class: "bg-gray-100 dark:bg-gray-900",
                crate::components::Header {}
            }
            main { class: "grow bg-white dark:bg-gray-900",
                ErrorBoundary {
                    handle_error: move |err: ErrorContext| {
                        let t = crate::i18n::use_translation();

                        #[cfg(feature = "server")]
                        let http_error = dioxus::fullstack::FullstackContext::commit_error_status(
                            err.error().unwrap(),
                        );

                        #[cfg(not(feature = "server"))]
                        let http_error = err.error().unwrap();

                        rsx! {
                            div { class: "text-red-500",
                                h1 { "{t.t(\"common-error\")}" }
                                p { "{http_error:?}" }
                            }
                        }
                    },
                    Outlet::<crate::app::Route> {}
                }
            }
            crate::components::Footer {}
        }
    }
}
