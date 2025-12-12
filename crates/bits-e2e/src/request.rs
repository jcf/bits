use crate::fixtures::TestContext;
use reqwest::{Client, RequestBuilder, Response};

pub struct Request {
    builder: RequestBuilder,
}

impl Request {
    pub fn host(mut self, host: &str) -> Self {
        self.builder = self.builder.header("Host", host);
        self
    }

    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.builder = self.builder.header(name, value);
        self
    }

    pub async fn send(self) -> Response {
        self.builder.send().await.expect("Request failed")
    }
}

#[must_use]
pub fn get(ctx: &TestContext, path: &str) -> Request {
    let builder = ctx.client.get(ctx.server.url(path));
    Request { builder }
}

#[must_use]
pub fn post(ctx: &TestContext, path: &str) -> Request {
    let builder = ctx.client.post(ctx.server.url(path));
    Request { builder }
}

#[must_use]
pub fn cookie_client() -> Client {
    Client::builder()
        .cookie_store(true)
        .build()
        .expect("Failed to build client")
}

pub async fn login(client: &Client, base_url: &str, email: &str, password: &str) {
    client
        .post(format!("{}/api/sessions", base_url))
        .header("requested-with", "bits/test")
        .header("content-type", "application/json")
        .json(&serde_json::json!({
            "email": email,
            "password": password
        }))
        .send()
        .await
        .expect("Login request failed");
}

pub async fn get_session(client: &Client, base_url: &str) -> Option<bits::User> {
    let response = client
        .get(format!("{}/api/session", base_url))
        .send()
        .await
        .expect("Session request failed");

    response.json().await.ok().flatten()
}
