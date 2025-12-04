use dioxus::server::axum::extract::Request;
use dioxus::server::axum::middleware::Next;
use dioxus::server::axum::response::Response;
use std::time::Instant;

/// Middleware to record HTTP request metrics
pub async fn track_metrics(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().to_string();
    let path = req.uri().path().to_string();

    let response = next.run(req).await;

    let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
    let status = response.status().as_u16();

    crate::metrics::record_http_request(&method, &path, status, duration_ms);

    response
}
