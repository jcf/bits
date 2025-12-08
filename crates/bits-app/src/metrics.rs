use metrics_exporter_prometheus::PrometheusHandle;
use std::sync::OnceLock;

static METRICS_HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthOutcome {
    Success,
    Failure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum AuthEvent {
    Login,
    Logout,
    Register,
}

/// Trait for events that can record themselves as metrics
pub trait RecordableEvent {
    fn record_outcome(&self, outcome: AuthOutcome);
}

/// Event type for login attempts
pub struct LoginAttempt;

impl RecordableEvent for LoginAttempt {
    fn record_outcome(&self, outcome: AuthOutcome) {
        record_auth_event(AuthEvent::Login, outcome);
    }
}

/// Event type for logout attempts
pub struct LogoutAttempt;

impl RecordableEvent for LogoutAttempt {
    fn record_outcome(&self, outcome: AuthOutcome) {
        record_auth_event(AuthEvent::Logout, outcome);
    }
}

/// Event type for registration attempts
pub struct RegisterAttempt;

impl RecordableEvent for RegisterAttempt {
    fn record_outcome(&self, outcome: AuthOutcome) {
        record_auth_event(AuthEvent::Register, outcome);
    }
}

/// Extension trait for Result that enables automatic metrics tracking
pub trait RecordMetrics<T, E> {
    fn record(self, event: impl RecordableEvent) -> Self;
}

impl<T, E> RecordMetrics<T, E> for Result<T, E> {
    fn record(self, event: impl RecordableEvent) -> Self {
        let outcome = if self.is_ok() {
            AuthOutcome::Success
        } else {
            AuthOutcome::Failure
        };
        event.record_outcome(outcome);
        self
    }
}

/// Initialize the Prometheus metrics recorder and return a handle for rendering metrics
pub fn init() -> PrometheusHandle {
    METRICS_HANDLE
        .get_or_init(|| {
            metrics_exporter_prometheus::PrometheusBuilder::new()
                .install_recorder()
                .expect("Failed to install Prometheus recorder")
        })
        .clone()
}

/// Record HTTP request metrics
pub fn record_http_request(method: &str, path: &str, status: u16, duration_ms: f64) {
    let method = method.to_string();
    let path = path.to_string();
    let status_str = status.to_string();
    metrics::counter!("http_requests_total", "method" => method, "path" => path, "status" => status_str).increment(1);
    metrics::histogram!("http_request_duration_ms").record(duration_ms);
}

/// Record authentication events
pub fn record_auth_event(event: AuthEvent, outcome: AuthOutcome) {
    let event = event.to_string();
    let success_str = match outcome {
        AuthOutcome::Success => "true",
        AuthOutcome::Failure => "false",
    };
    metrics::counter!("auth_events_total", "event" => event, "success" => success_str).increment(1);
}

/// Record database query metrics
pub fn record_db_query(operation: &str, duration_ms: f64) {
    let operation = operation.to_string();
    metrics::counter!("db_queries_total", "operation" => operation).increment(1);
    metrics::histogram!("db_query_duration_ms").record(duration_ms);
}

/// Record session operations
pub fn record_session_operation(operation: &str) {
    let operation = operation.to_string();
    metrics::counter!("session_operations_total", "operation" => operation).increment(1);
}

/// Record rate limit checks
pub fn record_rate_limit_check(endpoint: &str, result: &str) {
    let endpoint = endpoint.to_string();
    let result = result.to_string();
    metrics::counter!("rate_limit_checks_total", "endpoint" => endpoint, "result" => result)
        .increment(1);
}

/// Record rate limit hits (when limit is exceeded)
pub fn record_rate_limit_hit(limit_type: &str, endpoint: &str) {
    let limit_type = limit_type.to_string();
    let endpoint = endpoint.to_string();
    metrics::counter!("rate_limit_hits_total", "limit_type" => limit_type, "endpoint" => endpoint)
        .increment(1);
}
