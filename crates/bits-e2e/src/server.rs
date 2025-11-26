use anyhow::Result;
use std::net::SocketAddr;

pub struct TestServer {
    pub addr: SocketAddr,
}

impl TestServer {
    pub fn url(&self, path: &str) -> String {
        format!("http://{}{}", self.addr, path)
    }
}

pub async fn spawn_solo() -> Result<TestServer> {
    todo!("Implement solo server spawning")
}

pub async fn spawn_colo() -> Result<TestServer> {
    todo!("Implement colo server spawning")
}
