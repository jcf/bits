use crate::server::TestServer;
use anyhow::Result;
use bits_db::{PgTimestamp, PgUrl};
use sqlx::PgPool;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Tenant {
    pub id: i64,
    pub name: String,
    pub created_at: PgTimestamp,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct TenantDomain {
    pub id: i64,
    pub tenant_id: i64,
    pub domain: String,
    pub valid_from: PgTimestamp,
    pub valid_to: PgTimestamp,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct User {
    pub id: i64,
    pub created_at: PgTimestamp,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct EmailAddress {
    pub id: i64,
    pub user_id: i64,
    pub address: String,
    pub valid_from: PgTimestamp,
    pub valid_to: PgTimestamp,
}

pub struct TestContext {
    pub server: TestServer,
    pub db_pool: PgPool,
    pub client: reqwest::Client,
    pub argon2: argon2::Argon2<'static>,
}

impl TestContext {
    pub async fn create_user(&self, email: &str, password_hash: &str) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            "insert into users (password_hash) values ($1) returning id, created_at",
        )
        .bind(password_hash)
        .fetch_one(&self.db_pool)
        .await?;

        sqlx::query!(
            "insert into email_addresses (user_id, address) values ($1, $2)",
            user.id,
            email
        )
        .execute(&self.db_pool)
        .await?;

        Ok(user)
    }

    pub async fn verify_email(&self, user_id: i64) -> Result<()> {
        sqlx::query!(
            "insert into email_verifications (email_address_id)
             select id from email_addresses where user_id = $1",
            user_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn create_verified_user(&self, email: &str, password: &str) -> Result<(User, String)> {
        use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};

        let salt = SaltString::generate(&mut OsRng);
        let password_hash = self
            .argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?
            .to_string();

        let user = self.create_user(email, &password_hash).await?;
        self.verify_email(user.id).await?;

        Ok((user, password_hash))
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
        added_by: i64,
    ) -> Result<(Tenant, TenantDomain)> {
        let tenant = self.create_tenant(name).await?;

        let tenant_domain = sqlx::query_as::<_, TenantDomain>(
            "insert into tenant_domains (tenant_id, domain, added_by)
             values ($1, $2, $3)
             returning id, tenant_id, domain, valid_from, valid_to",
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

async fn create_test_database(base_url: &PgUrl) -> Result<(PgUrl, PgPool)> {
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
    sqlx::migrate!("../../migrations").run(&test_pool).await?;

    Ok((test_url, test_pool))
}

pub fn config() -> Result<bits_app::config::Config> {
    use std::env;

    // Try loading from env, fall back to test defaults if vars missing
    let mut config = bits_app::config::Config::from_env().unwrap_or_else(|_| {
        let database_url = env::var("DATABASE_URL_TEST")
            .or_else(|_| env::var("DATABASE_URL"))
            .unwrap_or_else(|_| "postgresql://bits:please@localhost:5432/bits_test".to_string())
            .parse()
            .expect("Invalid database URL");

        bits_app::config::Config {
            version: "test".to_string(),
            database_url,
            max_database_connections: 5,
            argon2_m_cost: 19456, // Use lower memory cost for faster tests (19 MiB)
            argon2_t_cost: 2,     // Use lower time cost for faster tests
            argon2_p_cost: 1,     // Single thread for simpler test environment
            master_key: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(), // Test key (64 bytes)
            port: 0,
            session_name: "b".to_string(),
            platform_domain: None,
            dangerously_allow_javascript_evaluation: false,
        }
    });

    // Override with DATABASE_URL_TEST if available
    if let Ok(test_db_url) = env::var("DATABASE_URL_TEST") {
        config = config.with_database_url(
            test_db_url
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid DATABASE_URL_TEST: {}", e))?,
        );
    }

    // Override port to 0 for parallel tests (let OS assign random port)
    Ok(config.with_port(0))
}

pub async fn setup_solo(config: bits_app::config::Config) -> Result<TestContext> {
    let (test_url, db_pool) = create_test_database(&config.database_url).await?;
    let test_config = config.clone().with_database_url(test_url);

    // Create Argon2 instance with same config as AppState
    let argon2_params = argon2::Params::new(
        config.argon2_m_cost,
        config.argon2_t_cost,
        config.argon2_p_cost,
        Some(argon2::Params::DEFAULT_OUTPUT_LEN),
    )
    .map_err(|e| anyhow::anyhow!("Invalid Argon2 parameters: {}", e))?;
    let argon2 = argon2::Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2_params,
    );

    let server = crate::server::spawn_solo(test_config).await?;
    let client = reqwest::Client::new();

    Ok(TestContext {
        server,
        db_pool,
        client,
        argon2,
    })
}

pub async fn setup_colo(config: bits_app::config::Config) -> Result<TestContext> {
    let (test_url, db_pool) = create_test_database(&config.database_url).await?;
    let test_config = config.clone().with_database_url(test_url);

    // Create Argon2 instance with same config as AppState
    let argon2_params = argon2::Params::new(
        config.argon2_m_cost,
        config.argon2_t_cost,
        config.argon2_p_cost,
        Some(argon2::Params::DEFAULT_OUTPUT_LEN),
    )
    .map_err(|e| anyhow::anyhow!("Invalid Argon2 parameters: {}", e))?;
    let argon2 = argon2::Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2_params,
    );

    let server = crate::server::spawn_colo(test_config).await?;
    let client = reqwest::Client::new();

    Ok(TestContext {
        server,
        db_pool,
        client,
        argon2,
    })
}
