use crate::server::TestServer;
use anyhow::Result;
use bits_db::PostgresUrl;
use sqlx::PgPool;

pub struct Tenant {
    pub id: i64,
}

pub struct TenantDomain {
    pub id: i64,
    pub tenant_id: i64,
    pub domain: String,
}

pub struct User {
    pub id: i64,
}

pub struct TestContext {
    pub server: TestServer,
    pub db_name: String,
    pub db_pool: PgPool,
    admin_pool: PgPool,
    runtime_handle: tokio::runtime::Handle,
}

impl TestContext {
    pub async fn create_user(&self, email: &str, password_hash: &str) -> Result<User> {
        let user = sqlx::query_as!(
            User,
            "INSERT INTO users (password_hash) VALUES ($1) RETURNING id",
            password_hash
        )
        .fetch_one(&self.db_pool)
        .await?;

        sqlx::query!(
            "INSERT INTO email_addresses (user_id, address) VALUES ($1, $2)",
            user.id,
            email
        )
        .execute(&self.db_pool)
        .await?;

        Ok(user)
    }

    pub async fn create_tenant(&self) -> Result<Tenant> {
        let tenant = sqlx::query_as!(
            Tenant,
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

        let tenant_domain = sqlx::query_as!(
            TenantDomain,
            "INSERT INTO tenant_domains (tenant_id, domain, added_by)
             VALUES ($1, $2, $3)
             RETURNING id, tenant_id, domain",
            tenant.id,
            domain,
            added_by
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok((tenant, tenant_domain))
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        let db_name = self.db_name.clone();
        let admin_pool = self.admin_pool.clone();

        let _ = self.runtime_handle.block_on(async move {
            let _ = sqlx::query(&format!(
                "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}'",
                db_name
            ))
            .execute(&admin_pool)
            .await;

            let _ = sqlx::query(&format!("DROP DATABASE IF EXISTS {} WITH (FORCE)", db_name))
                .execute(&admin_pool)
                .await;
        });
    }
}

async fn create_test_database(base_url: &PostgresUrl) -> Result<(String, PgPool, PgPool)> {
    let template_db = base_url
        .database()
        .ok_or_else(|| anyhow::anyhow!("DATABASE_URL missing database name"))?;

    let postgres_url = base_url.with_database("postgres");
    let admin_pool = PgPool::connect(postgres_url.as_ref()).await?;

    let test_id = uuid::Uuid::new_v4().simple().to_string();
    let db_name = format!("{}_{}", template_db, test_id);

    sqlx::query(&format!(
        "CREATE DATABASE {} TEMPLATE {}",
        db_name, template_db
    ))
    .execute(&admin_pool)
    .await?;

    let test_url = base_url.with_database(&db_name);
    let test_pool = PgPool::connect(test_url.as_ref()).await?;

    Ok((db_name, test_pool, admin_pool))
}

pub fn config() -> Result<bits_app::config::Config> {
    bits_app::config::Config::from_env()
        .map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))
}

pub async fn setup_solo(config: bits_app::config::Config) -> Result<TestContext> {
    let (db_name, db_pool, admin_pool) = create_test_database(&config.database_url).await?;
    let test_config = config.with_database_url(config.database_url.with_database(&db_name));

    let server = crate::server::spawn_solo(test_config).await?;
    let runtime_handle = tokio::runtime::Handle::current();

    Ok(TestContext {
        server,
        db_name,
        db_pool,
        admin_pool,
        runtime_handle,
    })
}

pub async fn setup_colo(config: bits_app::config::Config) -> Result<TestContext> {
    let (db_name, db_pool, admin_pool) = create_test_database(&config.database_url).await?;
    let test_config = config.with_database_url(config.database_url.with_database(&db_name));

    let server = crate::server::spawn_colo(test_config).await?;
    let runtime_handle = tokio::runtime::Handle::current();

    Ok(TestContext {
        server,
        db_name,
        db_pool,
        admin_pool,
        runtime_handle,
    })
}
