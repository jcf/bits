use reqwest::{header, Client};
use serde::Serialize;

pub struct BitsClient {
    client: Client,
    base_url: String,
}

impl BitsClient {
    pub fn new(base_url: String) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert("requested-with", header::HeaderValue::from_static("bits/test"));

        let client = Client::builder()
            .cookie_store(true)
            .default_headers(headers)
            .build()
            .expect("Failed to build client");

        Self { client, base_url }
    }

    pub async fn login(&self, email: &str, password: &str) -> reqwest::Response {
        let response = self
            .client
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

    pub async fn get_session(&self) -> Option<bits_app::User> {
        let response = self
            .client
            .get(format!("{}/api/session", self.base_url))
            .send()
            .await
            .expect("Session request failed");

        response.json().await.ok().flatten()
    }

    pub async fn change_password<T: Serialize>(
        &self,
        payload: &T,
    ) -> reqwest::Response {
        self.client
            .post(format!("{}/api/passwords", self.base_url))
            .json(payload)
            .send()
            .await
            .expect("Change password request failed")
    }
}
