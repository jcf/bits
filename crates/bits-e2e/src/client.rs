use reqwest::{header, Client, Response, StatusCode};
use scraper::{Html, Selector};
use serde::Serialize;
use std::sync::LazyLock;

static CSRF_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("meta[name='csrf-token']").unwrap());

#[derive(Debug)]
pub enum ClientError {
    Request(reqwest::Error),
    StatusCode(StatusCode),
    CsrfTokenMissing,
    ParseError(String),
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::Request(e) => write!(f, "Request failed: {}", e),
            ClientError::StatusCode(s) => write!(f, "HTTP {}", s),
            ClientError::CsrfTokenMissing => write!(f, "CSRF token not found"),
            ClientError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for ClientError {}

pub struct BitsClient {
    client: Client,
    base_url: String,
    csrf_token: Option<String>,
}

impl BitsClient {
    #[must_use]
    pub fn new(base_url: String) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "requested-with",
            header::HeaderValue::from_static("bits/test"),
        );

        let client = Client::builder()
            .cookie_store(true)
            .default_headers(headers)
            .build()
            .expect("Failed to build client");

        Self {
            client,
            base_url,
            csrf_token: None,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    async fn post_form_data(
        &self,
        path: &str,
        form: &[(&str, &str)],
    ) -> Result<Response, ClientError> {
        self.post(self.url(path))
            .form(form)
            .send()
            .await
            .map_err(ClientError::Request)
    }

    pub async fn fetch_csrf_token(&mut self) -> Result<(), ClientError> {
        let response = self
            .client
            .get(self.url("/"))
            .send()
            .await
            .map_err(ClientError::Request)?;

        let html = response.text().await.map_err(ClientError::Request)?;
        let document = Html::parse_document(&html);

        self.csrf_token = document
            .select(&CSRF_SELECTOR)
            .next()
            .and_then(|el| el.value().attr("content"))
            .map(String::from);

        self.csrf_token
            .as_ref()
            .ok_or(ClientError::CsrfTokenMissing)?;

        Ok(())
    }

    fn post(&self, url: String) -> reqwest::RequestBuilder {
        let mut builder = self.client.post(url);
        if let Some(token) = &self.csrf_token {
            builder = builder.header("csrf-token", token);
        }
        builder
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<(), ClientError> {
        let response = self
            .post_form_data("/api/sessions", &[("email", email), ("password", password)])
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(ClientError::StatusCode(response.status()))
        }
    }

    pub async fn join(&self, email: &str, password: &str) -> Result<(), ClientError> {
        let response = self
            .post_form_data("/api/users", &[("email", email), ("password", password)])
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(ClientError::StatusCode(response.status()))
        }
    }

    pub async fn logout(&self) -> Result<(), ClientError> {
        self.client
            .delete(self.url("/api/session"))
            .send()
            .await
            .map_err(ClientError::Request)?;
        Ok(())
    }

    pub async fn get_session(&self) -> Result<Option<bits_app::User>, ClientError> {
        let response = self
            .client
            .get(self.url("/api/session"))
            .send()
            .await
            .map_err(ClientError::Request)?;

        let session_state: bits_app::SessionState = response
            .json()
            .await
            .map_err(|e| ClientError::ParseError(e.to_string()))?;

        match session_state {
            bits_app::SessionState::Authenticated(user) => Ok(Some(user)),
            bits_app::SessionState::Anonymous => Ok(None),
        }
    }

    pub async fn change_password<T: Serialize>(
        &self,
        payload: &T,
    ) -> Result<Response, ClientError> {
        let response = self
            .post(self.url("/api/passwords"))
            .form(payload)
            .send()
            .await
            .map_err(ClientError::Request)?;
        Ok(response)
    }
}
