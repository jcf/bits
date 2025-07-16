use anyhow::Result;
use sqlx::{sqlite::SqlitePool, Row};
use std::path::Path;
use tracing::info;

#[derive(Clone)]
pub struct Storage {
    pool: SqlitePool,
}

impl Storage {
    pub async fn new(data_dir: &str) -> Result<Self> {
        // Create data directory if it doesn't exist
        std::fs::create_dir_all(data_dir)?;
        
        let db_path = Path::new(data_dir).join("node.db");
        let db_url = format!("sqlite:{}", db_path.display());
        
        // Create connection pool
        let pool = SqlitePool::connect(&db_url).await?;
        
        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;
        
        info!("Database initialized at: {}", db_path.display());
        
        Ok(Storage { pool })
    }

    pub async fn store_chunk(&self, key: &[u8], data: &[u8]) -> Result<()> {
        sqlx::query(
            "INSERT INTO chunks (key, data, created_at) VALUES (?, ?, datetime('now'))
             ON CONFLICT(key) DO UPDATE SET data = excluded.data, updated_at = datetime('now')"
        )
        .bind(key)
        .bind(data)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn get_chunk(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let result = sqlx::query("SELECT data FROM chunks WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;
        
        Ok(result.map(|row| row.get("data")))
    }

    pub async fn store_content_metadata(
        &self,
        cid: &str,
        creator: &str,
        size: i64,
        price: i64,
        encrypted_key: &[u8],
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO content (cid, creator, size, price, encrypted_key, created_at)
             VALUES (?, ?, ?, ?, ?, datetime('now'))"
        )
        .bind(cid)
        .bind(creator)
        .bind(size)
        .bind(price)
        .bind(encrypted_key)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn get_storage_stats(&self) -> Result<StorageStats> {
        let row = sqlx::query(
            "SELECT 
                COUNT(*) as chunk_count,
                SUM(LENGTH(data)) as total_size,
                COUNT(DISTINCT creator) as creator_count
             FROM chunks
             LEFT JOIN content ON 1=1"
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(StorageStats {
            chunk_count: row.get("chunk_count"),
            total_size: row.get("total_size"),
            creator_count: row.get("creator_count"),
        })
    }
}

#[derive(Debug)]
pub struct StorageStats {
    pub chunk_count: i64,
    pub total_size: i64,
    pub creator_count: i64,
}