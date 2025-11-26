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

    pub async fn send(self) -> Response {
        self.builder.send().await.expect("Request failed")
    }
}

pub fn get(ctx: &TestContext, path: &str) -> Request {
    let client = Client::new();
    let builder = client.get(ctx.server.url(path));
    Request { builder }
}

pub fn post(ctx: &TestContext, path: &str) -> Request {
    let client = Client::new();
    let builder = client.post(ctx.server.url(path));
    Request { builder }
}
