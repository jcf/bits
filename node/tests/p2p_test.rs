use anyhow::Result;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_network_initialization() -> Result<()> {
    // Start network on random port
    let network = node::p2p::Network::new(0, vec![]).await?;

    // Network should be created successfully
    // Just verify it doesn't panic
    sleep(Duration::from_millis(100)).await;

    Ok(())
}

#[tokio::test]
async fn test_multiple_nodes() -> Result<()> {
    // Start first node
    let _node1 = node::p2p::Network::new(0, vec![]).await?;

    // Start second node
    let _node2 = node::p2p::Network::new(0, vec![]).await?;

    // Give nodes time to discover each other via mDNS
    sleep(Duration::from_secs(2)).await;

    // TODO: Add peer discovery verification when implemented

    Ok(())
}

#[tokio::test]
async fn test_bootstrap_nodes() -> Result<()> {
    // Test with invalid bootstrap addresses
    let bootstrap = vec![
        "invalid_address".to_string(),
        "/ip4/127.0.0.1/tcp/9999".to_string(), // Valid format
    ];

    // Should handle invalid addresses gracefully
    let _network = node::p2p::Network::new(0, bootstrap).await?;

    sleep(Duration::from_millis(100)).await;

    Ok(())
}

#[tokio::test]
async fn test_content_publish() -> Result<()> {
    let network = node::p2p::Network::new(0, vec![]).await?;

    let cid = "QmTest123".to_string();
    let metadata = b"test metadata".to_vec();

    // Should not panic
    network.publish_content(cid, metadata).await?;

    Ok(())
}
