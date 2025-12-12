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

pub struct BitsClientBuilder {
    base_url: String,
    cookie_jar: std::sync::Arc<reqwest::cookie::Jar>,
    follow_redirects: bool,
}

impl BitsClientBuilder {
    #[must_use]
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            cookie_jar: std::sync::Arc::new(reqwest::cookie::Jar::default()),
            follow_redirects: true,
        }
    }

    pub fn cookie_jar(mut self, jar: std::sync::Arc<reqwest::cookie::Jar>) -> Self {
        self.cookie_jar = jar;
        self
    }

    #[must_use]
    pub fn follow_redirects(mut self, follow: bool) -> Self {
        self.follow_redirects = follow;
        self
    }

    #[must_use]
    pub fn build(self) -> BitsClient {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "requested-with",
            header::HeaderValue::from_static("bits/test"),
        );

        let mut builder = Client::builder()
            .cookie_provider(self.cookie_jar)
            .default_headers(headers);

        if !self.follow_redirects {
            builder = builder.redirect(reqwest::redirect::Policy::none());
        }

        let client = builder.build().expect("Failed to build client");

        BitsClient {
            client,
            base_url: self.base_url,
            csrf_token: None,
        }
    }
}

impl BitsClient {
    #[must_use]
    pub fn new(base_url: String) -> Self {
        BitsClientBuilder::new(base_url).build()
    }

    #[must_use]
    pub fn builder(base_url: String) -> BitsClientBuilder {
        BitsClientBuilder::new(base_url)
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

    fn patch(&self, url: String) -> reqwest::RequestBuilder {
        let mut builder = self.client.patch(url);
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

    pub async fn get_session(&self) -> Result<Option<bits::User>, ClientError> {
        let response = self
            .client
            .get(self.url("/api/session"))
            .send()
            .await
            .map_err(ClientError::Request)?;

        let session_state: bits::SessionState = response
            .json()
            .await
            .map_err(|e| ClientError::ParseError(e.to_string()))?;

        match session_state {
            bits::SessionState::Authenticated(user) => Ok(Some(user)),
            bits::SessionState::Anonymous => Ok(None),
        }
    }

    pub async fn change_password<T: Serialize>(
        &self,
        payload: &T,
    ) -> Result<Response, ClientError> {
        let response = self
            .patch(self.url("/api/passwords"))
            .form(payload)
            .send()
            .await
            .map_err(ClientError::Request)?;
        Ok(response)
    }

    pub fn get(&self, path: &str) -> reqwest::RequestBuilder {
        self.client.get(self.url(path))
    }
}
