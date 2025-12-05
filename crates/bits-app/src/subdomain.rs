use dioxus::prelude::*;

#[cfg(feature = "server")]
use dioxus::server::axum::extract::Extension;

#[cfg(feature = "server")]
use crate::tenant::Handle;
use crate::tenant::HandleError;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum SubdomainStatus {
    Available,
    Invalid(HandleError),
    Reserved,
    AlreadyTaken,
}

impl SubdomainStatus {
    /// Get the translation key for this status
    pub fn translation_key(&self) -> &'static str {
        match self {
            Self::Available => "subdomain-available",
            Self::Invalid(HandleError::TooShort { .. }) => "subdomain-invalid-too-short",
            Self::Invalid(HandleError::TooLong { .. }) => "subdomain-invalid-too-long",
            Self::Invalid(HandleError::MustStartWithLetter) => "subdomain-invalid-must-start-letter",
            Self::Invalid(HandleError::CannotEndWithHyphen) => "subdomain-invalid-hyphen-end",
            Self::Invalid(HandleError::ConsecutiveHyphens) => "subdomain-invalid-consecutive-hyphens",
            Self::Invalid(HandleError::InvalidCharacter(..)) => "subdomain-invalid-characters",
            Self::Invalid(HandleError::Reserved(_)) => "subdomain-reserved",
            Self::Reserved => "subdomain-reserved",
            Self::AlreadyTaken => "subdomain-already-taken",
        }
    }

    /// Check if this is an available status
    pub fn is_available(&self) -> bool {
        matches!(self, Self::Available)
    }

    /// Check if this is an error (invalid input)
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Invalid(_))
    }

    /// Check if this is a notice (reserved or taken)
    pub fn is_notice(&self) -> bool {
        matches!(self, Self::Reserved | Self::AlreadyTaken)
    }
}

#[derive(thiserror::Error, Debug, serde::Serialize, serde::Deserialize)]
pub enum SubdomainError {
    #[error("Internal error")]
    Internal(String),
}

impl From<ServerFnError> for SubdomainError {
    fn from(err: ServerFnError) -> Self {
        SubdomainError::Internal(err.to_string())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<sqlx::Error> for SubdomainError {
    fn from(err: sqlx::Error) -> Self {
        SubdomainError::Internal(err.to_string())
    }
}

impl dioxus::fullstack::AsStatusCode for SubdomainError {
    fn as_status_code(&self) -> dioxus::fullstack::StatusCode {
        dioxus::fullstack::StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[get("/api/handles/:handle", state: Extension<crate::AppState>)]
pub async fn check_subdomain(handle: String) -> Result<SubdomainStatus, SubdomainError> {
    // Validate handle
    let handle = match Handle::new(handle) {
        Ok(h) => h,
        Err(e) => return Ok(SubdomainStatus::Invalid(e)),
    };

    // Check if demo (reserved)
    if crate::demos::SUBDOMAINS.contains(&handle.as_str()) {
        return Ok(SubdomainStatus::Reserved);
    }

    // Check database for existing tenant
    if let Some(platform_domain) = &state.config.platform_domain {
        let domain = format!("{}.{}", handle, platform_domain);

        let exists = sqlx::query_scalar!(
            "select exists(
                select 1
                from tenant_domains
                where domain = $1
                and valid_to = 'infinity'
            )",
            domain
        )
        .fetch_one(&state.db)
        .await?
        .unwrap_or(false);

        if exists {
            return Ok(SubdomainStatus::AlreadyTaken);
        }
    }

    Ok(SubdomainStatus::Available)
}
