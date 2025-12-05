use reqwest::{header, Client};
use scraper::{Html, Selector};
use serde::Serialize;

pub struct BitsClient {
    client: Client,
    base_url: String,
    csrf_token: Option<String>,
}

impl BitsClient {
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

    pub async fn fetch_csrf_token(&mut self) {
        let response = self
            .client
            .get(&self.base_url)
            .send()
            .await
            .expect("Failed to load home page for CSRF token");

        let html = response.text().await.expect("Failed to read HTML");
        let document = Html::parse_document(&html);
        let selector = Selector::parse("meta[name='csrf-token']").unwrap();

        self.csrf_token = document
            .select(&selector)
            .next()
            .and_then(|el| el.value().attr("content"))
            .map(String::from);
    }

    fn post(&self, url: String) -> reqwest::RequestBuilder {
        let mut builder = self.client.post(url);
        if let Some(token) = &self.csrf_token {
            builder = builder.header("csrf-token", token);
        }
        builder
    }

    pub async fn login(&self, email: &str, password: &str) -> reqwest::Response {
        let response = self
            .post(format!("{}/api/sessions", self.base_url))
            .form(&[("email", email), ("password", password)])
            .send()
            .await
            .expect("Login request failed");

        if !response.status().is_success() {
            panic!(
                "Login failed with status {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
        }

        response
    }

    /// Login but return Result instead of panicking (for testing error cases)
    pub async fn login_result(&self, email: &str, password: &str) -> Result<(), u16> {
        let response = self
            .post(format!("{}/api/sessions", self.base_url))
            .form(&[("email", email), ("password", password)])
            .send()
            .await
            .expect("Login request failed");

        if response.status().is_success() {
            Ok(())
        } else {
            Err(response.status().as_u16())
        }
    }

    pub async fn join(&self, email: &str, password: &str) -> reqwest::Response {
        let response = self
            .post(format!("{}/api/users", self.base_url))
            .form(&[("email", email), ("password", password)])
            .send()
            .await
            .expect("Join request failed");

        if !response.status().is_success() {
            panic!(
                "Join failed with status {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
        }

        response
    }

    /// Join but return Result instead of panicking (for testing error cases)
    pub async fn join_result(&self, email: &str, password: &str) -> Result<(), u16> {
        let response = self
            .post(format!("{}/api/users", self.base_url))
            .form(&[("email", email), ("password", password)])
            .send()
            .await
            .expect("Join request failed");

        if response.status().is_success() {
            Ok(())
        } else {
            Err(response.status().as_u16())
        }
    }

    pub async fn logout(&self) {
        self.client
            .delete(format!("{}/api/session", self.base_url))
            .send()
            .await
            .expect("Logout request failed");
    }

    pub async fn get_session(&self) -> Option<bits_app::User> {
        let response = self
            .client
            .get(format!("{}/api/session", self.base_url))
            .send()
            .await
            .expect("Session request failed");

        let session_state: bits_app::SessionState = response.json().await.ok()?;
        match session_state {
            bits_app::SessionState::Authenticated(user) => Some(user),
            bits_app::SessionState::Anonymous => None,
        }
    }

    pub async fn change_password<T: Serialize>(&self, payload: &T) -> reqwest::Response {
        self.post(format!("{}/api/passwords", self.base_url))
            .form(payload)
            .send()
            .await
            .expect("Change password request failed")
    }
}
