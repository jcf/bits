use anyhow::Result;
use std::net::SocketAddr;
use tokio::net::TcpListener;

pub struct TestServer {
    pub addr: SocketAddr,
}

impl TestServer {
    pub fn url(&self, path: &str) -> String {
        format!("http://{}{}", self.addr, path)
    }
}

async fn spawn(router: axum::Router) -> Result<TestServer> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    tokio::spawn(async move {
        axum::serve(listener, router).await.ok();
    });

    Ok(TestServer { addr })
}

pub async fn spawn_solo(config: bits_app::Config) -> Result<TestServer> {
    let router = bits_solo::server::router(config).await?;
    spawn(router).await
}

pub async fn spawn_colo(config: bits_app::Config) -> Result<TestServer> {
    let router = bits_colo::server::router(config).await?;
    spawn(router).await
}
