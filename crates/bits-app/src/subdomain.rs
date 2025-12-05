use dioxus::prelude::*;

#[cfg(feature = "server")]
use dioxus::server::axum::extract::Extension;

#[cfg(feature = "server")]
use crate::tenant::{Handle, HandleError};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum SubdomainStatus {
    Available,
    Reserved(String),
    Invalid(String),
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
            return Ok(SubdomainStatus::Invalid("Must be 3-63 characters".into()));
        }
        Err(HandleError::InvalidCharacters) => {
            return Ok(SubdomainStatus::Invalid(
                "Only lowercase letters, numbers, and hyphens".into(),
            ));
        }
        Err(HandleError::InvalidFormat) => {
            return Ok(SubdomainStatus::Invalid(
                "Cannot start or end with hyphen".into(),
            ));
        }
    };

    // Check reserved system words
    match handle.as_str() {
        "www" | "api" | "app" | "admin" | "dashboard" | "cdn" | "assets" => {
            return Ok(SubdomainStatus::Reserved(
                "Reserved for platform use".into(),
            ));
        }
        _ => {}
    }

    // Easter eggs!
    match handle.as_str() {
        "god" => {
            return Ok(SubdomainStatus::Reserved(
                "Sorry, this one's taken by a higher power".into(),
            ));
        }
        "jesus" | "christ" => {
            return Ok(SubdomainStatus::Reserved(
                "He'll be back in three days to claim it".into(),
            ));
        }
        "satan" | "devil" | "lucifer" => {
            return Ok(SubdomainStatus::Reserved(
                "Already reserved in hell.bits.page".into(),
            ));
        }
        "nsfw" => {
            return Ok(SubdomainStatus::Reserved(
                "Too on the nose, try something creative!".into(),
            ));
        }
        "porn" | "xxx" | "sex" => {
            return Ok(SubdomainStatus::Reserved(
                "We get it, but be more subtle".into(),
            ));
        }
        "test" | "demo" | "example" | "sample" => {
            return Ok(SubdomainStatus::Reserved(
                "Reserved for testing and demos".into(),
            ));
        }
        "fuck" | "shit" | "damn" => {
            return Ok(SubdomainStatus::Reserved(
                "Creative profanity is an art. This isn't it.".into(),
            ));
        }
        _ => {}
    }

    // Check if demo
    if crate::demos::SUBDOMAINS.contains(&handle.as_str()) {
        return Ok(SubdomainStatus::Reserved("This is a demo profile".into()));
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
            return Ok(SubdomainStatus::Reserved("Already taken".into()));
        }
    }

    Ok(SubdomainStatus::Available)
}
