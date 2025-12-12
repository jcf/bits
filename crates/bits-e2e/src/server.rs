use anyhow::Result;
use std::net::SocketAddr;
use tokio::net::TcpListener;

pub struct TestServer {
    pub addr: SocketAddr,
}

impl TestServer {
    #[must_use]
    pub fn url(&self, path: &str) -> String {
        format!("http://{}{}", self.addr, path)
    }
}

fn init_tracing_once() {
    use std::sync::Once;
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
            )
            .with_test_writer()
            .init();
    });
}

async fn spawn(router: axum::Router) -> Result<TestServer> {
    init_tracing_once();

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    tokio::spawn(async move {
        axum::serve(listener, router).await.ok();
    });

    Ok(TestServer { addr })
}

pub async fn spawn_solo(config: bits::Config) -> Result<(TestServer, bits::AppState)> {
    let state = bits::init(config).await?;
    let router = bits::build_router(state.clone(), bits::App).await?;
    let server = spawn(router).await?;
    Ok((server, state))
}

pub async fn spawn_colo(config: bits::Config) -> Result<(TestServer, bits::AppState)> {
    let state = bits::init(config).await?;
    let router = bits::build_router(state.clone(), bits::App).await?;
    let server = spawn(router).await?;
    Ok((server, state))
}
