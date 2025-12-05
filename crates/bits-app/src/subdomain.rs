use dioxus::prelude::*;

#[cfg(feature = "server")]
use dioxus::server::axum::extract::Extension;

#[cfg(feature = "server")]
use crate::tenant::{Handle, HandleError};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum SubdomainStatus {
    Available,
    InvalidLength,
    InvalidCharacters,
    InvalidFormat,
    ReservedPlatform,
    ReservedGod,
    ReservedJesus,
    ReservedSatan,
    ReservedNsfw,
    ReservedProfanitySoft,
    ReservedTesting,
    ReservedProfanityCreative,
    ReservedDemo,
    AlreadyTaken,
}

impl SubdomainStatus {
    /// Get the translation key for this status
    pub fn translation_key(&self) -> &'static str {
        match self {
            Self::Available => "subdomain-available",
            Self::InvalidLength => "subdomain-invalid-length",
            Self::InvalidCharacters => "subdomain-invalid-characters",
            Self::InvalidFormat => "subdomain-invalid-format",
            Self::ReservedPlatform => "subdomain-reserved-platform",
            Self::ReservedGod => "subdomain-reserved-god",
            Self::ReservedJesus => "subdomain-reserved-jesus",
            Self::ReservedSatan => "subdomain-reserved-satan",
            Self::ReservedNsfw => "subdomain-reserved-nsfw",
            Self::ReservedProfanitySoft => "subdomain-reserved-profanity-soft",
            Self::ReservedTesting => "subdomain-reserved-testing",
            Self::ReservedProfanityCreative => "subdomain-reserved-profanity-creative",
            Self::ReservedDemo => "subdomain-reserved-demo",
            Self::AlreadyTaken => "subdomain-already-taken",
        }
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
    // Try to create validated Handle
    let handle = match Handle::new(handle) {
        Ok(h) => h,
        Err(HandleError::InvalidLength) => {
            return Ok(SubdomainStatus::InvalidLength);
        }
        Err(HandleError::InvalidCharacters) => {
            return Ok(SubdomainStatus::InvalidCharacters);
        }
        Err(HandleError::InvalidFormat) => {
            return Ok(SubdomainStatus::InvalidFormat);
        }
    };

    // Check reserved system words
    match handle.as_str() {
        "www" | "api" | "app" | "admin" | "dashboard" | "cdn" | "assets" => {
            return Ok(SubdomainStatus::ReservedPlatform);
        }
        _ => {}
    }

    // Easter eggs!
    match handle.as_str() {
        "god" => {
            return Ok(SubdomainStatus::ReservedGod);
        }
        "jesus" | "christ" => {
            return Ok(SubdomainStatus::ReservedJesus);
        }
        "satan" | "devil" | "lucifer" => {
            return Ok(SubdomainStatus::ReservedSatan);
        }
        "nsfw" => {
            return Ok(SubdomainStatus::ReservedNsfw);
        }
        "porn" | "xxx" | "sex" => {
            return Ok(SubdomainStatus::ReservedProfanitySoft);
        }
        "test" | "demo" | "example" | "sample" => {
            return Ok(SubdomainStatus::ReservedTesting);
        }
        "fuck" | "shit" | "damn" => {
            return Ok(SubdomainStatus::ReservedProfanityCreative);
        }
        _ => {}
    }

    // Check if demo
    if crate::demos::SUBDOMAINS.contains(&handle.as_str()) {
        return Ok(SubdomainStatus::ReservedDemo);
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
