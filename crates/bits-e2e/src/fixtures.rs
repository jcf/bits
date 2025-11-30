use crate::server::TestServer;
use anyhow::Result;
use bits_db::PostgresUrl;
use sqlx::PgPool;

#[derive(sqlx::FromRow)]
pub struct Tenant {
    pub id: i64,
}

#[derive(sqlx::FromRow)]
pub struct TenantDomain {
    pub id: i64,
    pub tenant_id: i64,
    pub domain: String,
}

#[derive(sqlx::FromRow)]
pub struct User {
    pub id: i64,
}

pub struct TestContext {
    pub server: TestServer,
    pub db_pool: PgPool,
    pub client: reqwest::Client,
}

impl TestContext {
    pub async fn create_user(&self, email: &str, password_hash: &str) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (password_hash) VALUES ($1) RETURNING id"
        )
        .bind(password_hash)
        .fetch_one(&self.db_pool)
        .await?;

        sqlx::query(
            "INSERT INTO email_addresses (user_id, address) VALUES ($1, $2)"
        )
        .bind(user.id)
        .bind(email)
        .execute(&self.db_pool)
        .await?;

        Ok(user)
    }

    pub async fn create_tenant(&self) -> Result<Tenant> {
        let tenant = sqlx::query_as::<_, Tenant>(
            "INSERT INTO tenants DEFAULT VALUES RETURNING id"
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(tenant)
    }

    pub async fn create_tenant_with_domain(
        &self,
        domain: &str,
        added_by: i64,
    ) -> Result<(Tenant, TenantDomain)> {
        let tenant = self.create_tenant().await?;

        let tenant_domain = sqlx::query_as::<_, TenantDomain>(
            "INSERT INTO tenant_domains (tenant_id, domain, added_by)
             VALUES ($1, $2, $3)
             RETURNING id, tenant_id, domain"
        )
        .bind(tenant.id)
        .bind(domain)
        .bind(added_by)
        .fetch_one(&self.db_pool)
        .await?;

        Ok((tenant, tenant_domain))
    }
}

// Note: We don't auto-cleanup test databases in Drop to avoid panics during shutdown
// and to allow debugging failed tests. Clean up manually with:
//   psql -U bits -c "DROP DATABASE bits_test_*" postgres

async fn create_test_database(base_url: &PostgresUrl) -> Result<(PostgresUrl, PgPool)> {
    use sqlx::postgres::PgPoolOptions;

    let postgres_url = base_url.with_database("postgres");
    let admin_pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(postgres_url.as_ref())
        .await?;

    let test_id = uuid::Uuid::new_v4().simple().to_string();
    let db_name = format!("bits_test_{}", test_id);

    sqlx::query(&format!("CREATE DATABASE {}", db_name))
        .execute(&admin_pool)
        .await?;

    let test_url = base_url.with_database(&db_name);
    let test_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(test_url.as_ref())
        .await?;

    // Run migrations
    sqlx::migrate!("../../migrations")
        .run(&test_pool)
        .await?;

    Ok((test_url, test_pool))
}

pub fn config() -> Result<bits_app::config::Config> {
    use std::env;

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://bits:please@localhost:5432/bits_test".to_string())
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid DATABASE_URL: {}", e))?;

    let max_database_connections = env::var("MAX_DATABASE_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);

    let port = env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0); // Use 0 to let the OS assign a random port

    // bits-e2e always enables colo feature on bits-app when server is enabled
    let config = bits_app::config::Config {
        database_url,
        max_database_connections,
        port,
        platform_domain: env::var("PLATFORM_DOMAIN").ok(),
    };

    Ok(config)
}

pub async fn setup_solo(config: bits_app::config::Config) -> Result<TestContext> {
    let (test_url, db_pool) = create_test_database(&config.database_url).await?;
    let test_config = config.with_database_url(test_url);

    let server = crate::server::spawn_solo(test_config).await?;
    let client = reqwest::Client::new();

    Ok(TestContext {
        server,
        db_pool,
        client,
    })
}

pub async fn setup_colo(config: bits_app::config::Config) -> Result<TestContext> {
    let (test_url, db_pool) = create_test_database(&config.database_url).await?;
    let test_config = config.with_database_url(test_url);

    let server = crate::server::spawn_colo(test_config).await?;
    let client = reqwest::Client::new();

    Ok(TestContext {
        server,
        db_pool,
        client,
    })
}
