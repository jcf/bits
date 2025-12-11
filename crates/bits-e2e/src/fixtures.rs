use crate::server::TestServer;
use anyhow::Result;
use bits_db::{PgTimestamp, PgUrl};
use sqlx::PgPool;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Tenant {
    pub id: bits_domain::TenantId,
    pub name: String,
    pub created_at: PgTimestamp,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct TenantDomain {
    pub id: i64,
    pub tenant_id: bits_domain::TenantId,
    pub domain: String,
    pub valid_from: PgTimestamp,
    pub valid_to: PgTimestamp,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct User {
    pub id: bits_domain::UserId,
    pub created_at: PgTimestamp,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct EmailAddress {
    pub id: bits_domain::EmailAddressId,
    pub user_id: bits_domain::UserId,
    pub address: bits_domain::Email,
    pub valid_from: PgTimestamp,
    pub valid_to: PgTimestamp,
}

pub struct TestContext {
    pub server: TestServer,
    pub db_pool: PgPool,
    pub client: reqwest::Client,
    pub state: bits_app::AppState,
    test_db_name: String,
}

impl TestContext {
    pub async fn create_user(
        &self,
        email: &str,
        password_hash: &bits_domain::PasswordHash,
    ) -> Result<User> {
        use secrecy::ExposeSecret;

        let user = sqlx::query_as::<_, User>(
            "insert into users (password_hash) values ($1) returning id, created_at",
        )
        .bind(password_hash.expose_secret())
        .fetch_one(&self.db_pool)
        .await?;

        let email_domain = bits_domain::Email::parse(email)?;
        sqlx::query!(
            "insert into email_addresses (user_id, address) values ($1, $2)",
            user.id.get(),
            email_domain.as_str()
        )
        .execute(&self.db_pool)
        .await?;

        Ok(user)
    }

    pub async fn verify_email(&self, user_id: bits_domain::UserId) -> Result<()> {
        sqlx::query!(
            "insert into email_verifications (email_address_id)
             select id from email_addresses where user_id = $1",
            user_id.get()
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn create_verified_user(
        &self,
        email: &str,
        password: &str,
    ) -> Result<(User, bits_domain::PasswordHash)> {
        let password_domain = bits_domain::Password::new(password.to_string());
        let password_hash = self
            .state
            .password_service
            .hash_password(&password_domain)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;

        let user = self.create_user(email, &password_hash).await?;
        self.verify_email(user.id).await?;

        Ok((user, password_hash))
    }

    pub async fn get_email_address_id(
        &self,
        user_id: bits_domain::UserId,
    ) -> Result<bits_domain::EmailAddressId> {
        let id = sqlx::query_scalar::<_, bits_domain::EmailAddressId>(
            "select id from email_addresses where user_id = $1",
        )
        .bind(user_id.get())
        .fetch_one(&self.db_pool)
        .await?;

        Ok(id)
    }

    pub async fn create_tenant(&self, name: &str) -> Result<Tenant> {
        let tenant = sqlx::query_as::<_, Tenant>(
            "insert into tenants (name) values ($1) returning id, name, created_at",
        )
        .bind(name)
        .fetch_one(&self.db_pool)
        .await?;

        Ok(tenant)
    }

    pub async fn create_tenant_with_domain(
        &self,
        name: &str,
        domain: &str,
        added_by: bits_domain::UserId,
    ) -> Result<(Tenant, TenantDomain)> {
        let tenant = self.create_tenant(name).await?;

        let tenant_domain = sqlx::query_as::<_, TenantDomain>(
            "insert into tenant_domains (tenant_id, domain, added_by)
             values ($1, $2, $3)
             returning id, tenant_id, domain, valid_from, valid_to",
        )
        .bind(tenant.id.get())
        .bind(domain)
        .bind(added_by.get())
        .fetch_one(&self.db_pool)
        .await?;

        Ok((tenant, tenant_domain))
    }

    pub async fn mark_tenant_as_fallback(&self, tenant_id: bits_domain::TenantId) -> Result<()> {
        // Clear any existing fallback
        sqlx::query!("update tenants set is_fallback = false")
            .execute(&self.db_pool)
            .await?;

        // Mark specified tenant as fallback
        sqlx::query!(
            "update tenants set is_fallback = true where id = $1",
            tenant_id.get()
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        let db_name = self.test_db_name.clone();
        let base_url = self.state.config.database_url.clone();

        // Spawn thread with its own runtime since we can't block the test's runtime
        let handle = std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async move {
                if let Err(e) = cleanup_test_database(&base_url, &db_name).await {
                    tracing::warn!(
                        database = %db_name,
                        error = %e,
                        "Failed to cleanup test database"
                    );
                }
            })
        });

        handle.join().ok();
    }
}

async fn cleanup_test_database(base_url: &PgUrl, db_name: &str) -> Result<()> {
    use sqlx::postgres::PgPoolOptions;

    tracing::debug!(database = %db_name, "Cleaning up test database");

    let postgres_url = base_url.with_database("postgres");
    let admin_pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(postgres_url.as_ref())
        .await?;

    // Terminate active connections to the test database
    sqlx::query(
        "select pg_terminate_backend(pid)
         from pg_stat_activity
         where datname = $1 and pid <> pg_backend_pid()",
    )
    .bind(db_name)
    .execute(&admin_pool)
    .await?;

    // Drop the database using identifier quoting
    // PostgreSQL doesn't support parameterized identifiers, so we use quote_ident
    sqlx::query(&format!("drop database if exists {}", quote_ident(db_name)))
        .execute(&admin_pool)
        .await?;

    tracing::debug!(database = %db_name, "Test database dropped");

    Ok(())
}

/// Quote a PostgreSQL identifier (database name, table name, etc.)
/// Prevents SQL injection by escaping quotes and wrapping in double quotes
fn quote_ident(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

async fn create_test_database(base_url: &PgUrl) -> Result<(PgUrl, String)> {
    use sqlx::postgres::PgPoolOptions;

    let test_id = uuid::Uuid::new_v4().simple().to_string();
    let db_name = format!("bits_test_{}", test_id);

    tracing::debug!(database = %db_name, "Creating test database");

    let postgres_url = base_url.with_database("postgres");
    let admin_pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(postgres_url.as_ref())
        .await?;

    // Create database using identifier quoting
    sqlx::query(&format!("create database {}", quote_ident(&db_name)))
        .execute(&admin_pool)
        .await?;

    tracing::debug!(database = %db_name, "Running migrations");

    let test_url = base_url.with_database(&db_name);
    let test_pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(test_url.as_ref())
        .await?;
    sqlx::migrate!("../../migrations").run(&test_pool).await?;

    tracing::debug!(database = %db_name, "Test database ready");

    Ok((test_url, db_name))
}

pub fn config() -> Result<bits_app::config::Config> {
    use std::env;

    let database_url = env::var("DATABASE_URL_TEST")
        .or_else(|_| env::var("DATABASE_URL"))
        .unwrap_or_else(|_| "postgresql://bits:please@localhost:5432/bits_test".to_string())
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid database URL: {}", e))?;

    let mut config = bits_app::load_config()?
        .with_database_url(database_url)
        .with_port(0)
        .with_test_argon2_params();

    config.global_rate_limit = None;

    Ok(config)
}

pub async fn setup_solo(config: bits_app::config::Config) -> Result<TestContext> {
    let (test_url, test_db_name) = create_test_database(&config.database_url).await?;
    let test_config = config.clone().with_database_url(test_url.clone());

    let (server, state) = crate::server::spawn_solo(test_config).await?;
    let client = reqwest::Client::new();

    let ctx = TestContext {
        server,
        db_pool: state.db.clone(),
        client,
        state,
        test_db_name,
    };

    // Create a default tenant and mark it as fallback for solo mode
    let tenant = ctx.create_tenant("Default Tenant").await?;
    ctx.mark_tenant_as_fallback(tenant.id).await?;

    Ok(ctx)
}

pub async fn setup_colo(config: bits_app::config::Config) -> Result<TestContext> {
    let (test_url, test_db_name) = create_test_database(&config.database_url).await?;
    let test_config = config.clone().with_database_url(test_url.clone());

    let (server, state) = crate::server::spawn_colo(test_config).await?;
    let client = reqwest::Client::new();

    Ok(TestContext {
        server,
        db_pool: state.db.clone(),
        client,
        state,
        test_db_name,
    })
}
