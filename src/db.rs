use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::{Mutex, OnceCell};

/// Global database connection pool.
static POOL_CELL: OnceCell<PgPool> = OnceCell::const_new();

/// Initialize and return the database pool.
pub async fn pool() -> &'static PgPool {
    POOL_CELL
        .get_or_init(|| async {
            let database_url =
                std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

            PgPoolOptions::new()
                .max_connections(5)
                .connect(&database_url)
                .await
                .expect("Failed to connect to database")
        })
        .await
}

/// A database connection with tenant schema already configured.
#[derive(Clone)]
pub struct TenantDb(pub Arc<Mutex<sqlx::pool::PoolConnection<sqlx::Postgres>>>);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_connects() {
        let pool = pool().await;
        let row: (i32,) = sqlx::query_as("SELECT 1")
            .fetch_one(pool)
            .await
            .expect("Failed to execute query");
        assert_eq!(row.0, 1);
    }
}
