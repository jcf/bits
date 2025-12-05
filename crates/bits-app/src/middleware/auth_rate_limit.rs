//! Middleware for authentication endpoint rate limiting

use crate::auth_rate_limit::{AuthRateLimitService, RateLimitError};
use crate::AppState;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::header::HeaderMap;
use axum::middleware::Next;
use axum::response::Response;
use bytes::Bytes;
use http_body_util::BodyExt;
use std::net::IpAddr;

/// Extract client IP address from request headers
///
/// Priority order:
/// 1. CF-Connecting-IP (Cloudflare)
/// 2. X-Forwarded-For (load balancers, proxies)
/// 3. X-Real-IP (nginx, other proxies)
/// 4. Fallback to localhost (should be set by RealIpLayer in production)
fn extract_client_ip(headers: &HeaderMap) -> IpAddr {
    headers
        .get("cf-connecting-ip")
        .or_else(|| headers.get("x-forwarded-for"))
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .and_then(|s| {
            // X-Forwarded-For can be comma-separated list, take first IP
            s.split(',').next().map(|ip| ip.trim())
        })
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)))
}

/// Extract email from URL-encoded form body
///
/// This matches the format used by Dioxus forms (application/x-www-form-urlencoded).
/// Returns None if body doesn't contain email field.
fn extract_email_from_body(body: &Bytes) -> Option<String> {
    let body_str = std::str::from_utf8(body).ok()?;

    // Parse URL-encoded form data
    for pair in body_str.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            if key == "email" {
                return urlencoding::decode(value).ok().map(|s| s.into_owned());
            }
        }
    }

    None
}

/// Middleware for rate limiting authentication endpoints
///
/// Checks rate limits based on IP and email, then records the attempt.
/// Returns early with error response if limits are exceeded.
pub async fn auth_rate_limit_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, RateLimitError> {
    let path = request.uri().path().to_string();
    let method = request.method().clone();

    // Only apply to authentication endpoints
    let is_login = path == "/api/sessions" && method == "POST";
    let is_registration = path == "/api/users" && method == "POST";

    if !is_login && !is_registration {
        // Not an auth endpoint, skip rate limiting
        return Ok(next.run(request).await);
    }

    // Extract client IP from headers (RealIpLayer should have set these)
    let client_ip = extract_client_ip(request.headers());

    // Read request body to extract email
    let (parts, body) = request.into_parts();
    let body_bytes = body
        .collect()
        .await
        .map_err(|e| RateLimitError::Internal(format!("Failed to read request body: {}", e)))?
        .to_bytes();

    // Extract email from body
    let email = extract_email_from_body(&body_bytes)
        .ok_or_else(|| RateLimitError::Internal("Missing email in request body".to_string()))?;

    // Check rate limits
    if is_login {
        state
            .auth_rate_limit
            .check_login_limits(&state.db, client_ip, &email)
            .await?;
    } else if is_registration {
        state
            .auth_rate_limit
            .check_registration_limits(&state.db, client_ip, &email)
            .await?;
    }

    // Record metrics
    crate::metrics::record_rate_limit_check(&path, "passed");

    // Reconstruct request with body
    let request = Request::from_parts(parts, Body::from(body_bytes));

    // Continue to handler
    let response = next.run(request).await;

    // Record attempt after handler completes (regardless of auth success/failure)
    // This happens in background to not slow down response
    let state_clone = state.clone();
    let email_clone = email.clone();
    let path_clone = path.to_string();
    tokio::spawn(async move {
        if let Err(e) = state_clone
            .auth_rate_limit
            .record_attempt(&state_clone.db, client_ip, &email_clone, &path_clone)
            .await
        {
            tracing::error!(
                error = %e,
                ip = %client_ip,
                email = %email_clone,
                endpoint = %path_clone,
                "Failed to record auth attempt"
            );
        }
    });

    Ok(response)
}
