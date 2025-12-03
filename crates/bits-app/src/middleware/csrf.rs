use crate::AppState;
use dioxus::server::axum::body::Body;
use dioxus::server::axum::http::{header, Method, StatusCode};
use dioxus::server::axum::{extract::Request, response::Response};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{Layer, Service};

fn is_side_effectful(method: &Method) -> bool {
    matches!(
        *method,
        Method::POST | Method::PUT | Method::DELETE | Method::PATCH
    )
}

fn extract_session_id(req: &Request, session_name: &str) -> Option<String> {
    let cookie_prefix = format!("{}=", session_name);
    req.headers()
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies
                .split(';')
                .find(|part| part.trim().starts_with(&cookie_prefix))
                .and_then(|s| s.trim().strip_prefix(&cookie_prefix))
                .map(|s| s.to_string())
        })
}

fn extract_csrf_token_from_header(req: &Request) -> Option<String> {
    req.headers()
        .get("csrf-token")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

async fn extract_csrf_token_from_form(req: &mut Request) -> Option<String> {
    use http_body_util::BodyExt;

    // Check if content-type is form-urlencoded
    let content_type = req
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())?;

    if !content_type.contains("application/x-www-form-urlencoded") {
        return None;
    }

    // Read the body
    let mut body = std::mem::replace(req.body_mut(), Body::empty());
    let mut bytes = Vec::new();

    while let Some(frame) = body.frame().await {
        if let Ok(frame) = frame {
            if let Some(data) = frame.data_ref() {
                bytes.extend_from_slice(data);
            }
        }
    }

    // Parse form data
    let form_data = String::from_utf8_lossy(&bytes);
    let mut csrf_token = None;
    let mut other_fields = Vec::new();

    for pair in form_data.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            if key == "csrf_token" {
                csrf_token = Some(urlencoding::decode(value).ok()?.to_string());
            } else {
                other_fields.push(pair);
            }
        } else {
            other_fields.push(pair);
        }
    }

    // Reconstruct body without csrf_token field
    let new_body = other_fields.join("&");
    *req.body_mut() = Body::from(new_body);

    csrf_token
}

#[derive(Clone)]
pub struct CsrfVerificationLayer;

impl<S> Layer<S> for CsrfVerificationLayer {
    type Service = CsrfVerificationMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CsrfVerificationMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct CsrfVerificationMiddleware<S> {
    inner: S,
}

impl<S> Service<Request> for CsrfVerificationMiddleware<S>
where
    S: Service<Request, Response = Response> + Send + Clone + 'static,
    S::Future: Send,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Response, S::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request) -> Self::Future {
        let mut inner = self.inner.clone();

        Box::pin(async move {
            if !is_side_effectful(req.method()) {
                return inner.call(req).await;
            }

            // Extract analytics header for logging (not used for access control)
            let requested_with = req
                .headers()
                .get("requested-with")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("unknown")
                .to_string();

            // Get AppState to access session name from config
            let state = match req.extensions().get::<AppState>() {
                Some(state) => state,
                None => {
                    tracing::error!("CSRF verification failed: AppState not found in extensions");
                    return Ok(Response::builder()
                        .status(StatusCode::FORBIDDEN)
                        .body(Body::from("Forbidden.\n"))
                        .unwrap());
                }
            };

            let session_id = match extract_session_id(&req, &state.config.session_name) {
                Some(id) => id,
                None => {
                    tracing::warn!(
                        "CSRF verification failed: no session cookie (method: {}, path: {}, client: {})",
                        req.method(),
                        req.uri().path(),
                        requested_with
                    );
                    return Ok(Response::builder()
                        .status(StatusCode::FORBIDDEN)
                        .body(Body::from("Forbidden.\n"))
                        .unwrap());
                }
            };

            tracing::debug!(
                "CSRF verification for {} {} (session: {}, client: {})",
                req.method(),
                req.uri().path(),
                session_id,
                requested_with
            );

            // Try header first, then form parameter
            let provided_token = match extract_csrf_token_from_header(&req) {
                Some(token) => {
                    tracing::debug!("CSRF token found in header");
                    Some(token)
                }
                None => {
                    let token = extract_csrf_token_from_form(&mut req).await;
                    if token.is_some() {
                        tracing::debug!("CSRF token found in form parameter");
                    }
                    token
                }
            };

            let provided_token = match provided_token {
                Some(token) => token,
                None => {
                    tracing::warn!(
                        "CSRF verification failed: no token provided (session: {}, client: {})",
                        session_id,
                        requested_with
                    );
                    return Ok(Response::builder()
                        .status(StatusCode::FORBIDDEN)
                        .body(Body::from("Forbidden.\n"))
                        .unwrap());
                }
            };

            // Validate token format
            if !crate::csrf::is_valid_format(&provided_token) {
                tracing::warn!(
                    "CSRF verification failed: invalid token format (session: {}, client: {})",
                    session_id,
                    requested_with
                );
                return Ok(Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .body(Body::from("Forbidden.\n"))
                    .unwrap());
            }

            // Get session from extensions to check token in memory
            let session = match req.extensions().get::<axum_session::Session<bits_axum_session_sqlx::SessionPgPool>>() {
                Some(s) => s,
                None => {
                    tracing::error!("CSRF verification failed: Session not found in extensions");
                    return Ok(Response::builder()
                        .status(StatusCode::FORBIDDEN)
                        .body(Body::from("Forbidden.\n"))
                        .unwrap());
                }
            };

            // Get expected token from session memory
            let expected_token = match session.get::<String>("csrf_token") {
                Some(token) => token,
                None => {
                    tracing::warn!(
                        "CSRF verification failed: no token in session (session: {}, client: {})",
                        session_id,
                        requested_with
                    );
                    return Ok(Response::builder()
                        .status(StatusCode::FORBIDDEN)
                        .body(Body::from("Forbidden.\n"))
                        .unwrap());
                }
            };

            // Verify token using timing-safe comparison
            if !crate::csrf::verify_token(&expected_token, &provided_token) {
                tracing::warn!(
                    "CSRF verification failed: token mismatch (session: {}, client: {})",
                    session_id,
                    requested_with
                );
                return Ok(Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .body(Body::from("Forbidden.\n"))
                    .unwrap());
            }

            // Token is valid - allow request
            // Note: We keep the token for session lifetime to support multi-tab UX
            // and distributed systems. Same-session replays are not a security threat.
            tracing::debug!(
                "CSRF verification passed (session: {}, client: {})",
                session_id,
                requested_with
            );

            inner.call(req).await
        })
    }
}
