use anyhow::Result;
use tempfile::TempDir;

#[tokio::test]
async fn test_storage_initialization() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let storage = node::storage::Storage::new(temp_dir.path().to_str().unwrap()).await?;

    // Check that database file was created
    assert!(temp_dir.path().join("node.db").exists());

    Ok(())
}

#[tokio::test]
async fn test_chunk_storage() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let storage = node::storage::Storage::new(temp_dir.path().to_str().unwrap()).await?;

    let key = b"test_key";
    let data = b"test_data";

    // Store chunk
    storage.store_chunk(key, data).await?;

    // Retrieve chunk
    let retrieved = storage.get_chunk(key).await?;
    assert_eq!(retrieved, Some(data.to_vec()));

    // Non-existent chunk
    let missing = storage.get_chunk(b"missing_key").await?;
    assert_eq!(missing, None);

    Ok(())
}

#[tokio::test]
async fn test_content_metadata() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let storage = node::storage::Storage::new(temp_dir.path().to_str().unwrap()).await?;

    let cid = "QmTest123";
    let creator = "did:example:creator123";
    let encrypted_key = b"encrypted_key_data";

    storage
        .store_content_metadata(
            cid,
            creator,
            1024, // size
            100,  // price
            encrypted_key,
        )
        .await?;

    // TODO: Add retrieval test when implemented

    Ok(())
}

#[tokio::test]
async fn test_storage_stats() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let storage = node::storage::Storage::new(temp_dir.path().to_str().unwrap()).await?;

    // Store some chunks
    for i in 0..5 {
        let key = format!("key_{}", i);
        let data = format!("data_{}", i);
        storage.store_chunk(key.as_bytes(), data.as_bytes()).await?;
    }

    let stats = storage.get_storage_stats().await?;
    assert_eq!(stats.chunk_count, 5);
    assert!(stats.total_size > 0);

    Ok(())
}
