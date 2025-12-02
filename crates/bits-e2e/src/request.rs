use crate::fixtures::TestContext;
use reqwest::{RequestBuilder, Response};

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

pub fn get(ctx: &TestContext, path: &str) -> Request {
    let builder = ctx.client.get(ctx.server.url(path));
    Request { builder }
}

pub fn post(ctx: &TestContext, path: &str) -> Request {
    let builder = ctx.client.post(ctx.server.url(path));
    Request { builder }
}
