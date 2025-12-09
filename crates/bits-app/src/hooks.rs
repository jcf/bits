use dioxus::prelude::*;

use crate::auth::SessionState;

#[cfg(feature = "server")]
use axum::http::{header, HeaderValue};

/// Redirects authenticated users away from public pages (auth/join) to home.
///
/// On the server, sends a 302 redirect. On the client, uses navigator.
///
/// Returns None if the user should be redirected (caller should return early),
/// or Some(()) if rendering should continue.
#[must_use]
pub fn use_redirect_if_authenticated() -> Option<()> {
    let session = use_context::<Resource<Result<SessionState>>>();
    let nav = navigator();

    match session() {
        Some(Ok(SessionState::Authenticated(_))) => {
            #[cfg(feature = "server")]
            {
                if let Some(ctx) = dioxus::fullstack::FullstackContext::current() {
                    ctx.add_response_header(header::LOCATION, HeaderValue::from_static("/"));
                    dioxus::fullstack::FullstackContext::commit_http_status(
                        dioxus::fullstack::StatusCode::FOUND,
                        None,
                    );
                }
            }

            nav.push(crate::app::Route::Home {});
            None
        }
        _ => Some(()),
    }
}
