use crate::fixtures::{EmailAddress, Tenant, TenantDomain, User};
use anyhow::{Context, Result};
use serde::Deserialize;
use sqlx::PgPool;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Seeds {
    #[serde(default)]
    pub tenant: Vec<SeedTenant>,
    #[serde(default)]
    pub user: Vec<SeedUser>,
}

#[derive(Debug, Deserialize)]
pub struct SeedTenant {
    pub name: String,
    pub domain: String,
}

#[derive(Debug, Deserialize)]
pub struct SeedUser {
    pub email: String,
    pub password: String,
}

pub fn load_seeds(path: impl AsRef<Path>) -> Result<Seeds> {
    let content = std::fs::read_to_string(path).context("Failed to read seeds file")?;
    toml::from_str(&content).context("Failed to parse seeds TOML")
}

fn hash_password(argon2: &argon2::Argon2<'_>, password: &str) -> Result<String> {
    use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};

    let salt = SaltString::generate(&mut OsRng);
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?
        .to_string();

    Ok(hash)
}

async fn seed_user(
    pool: &PgPool,
    argon2: &argon2::Argon2<'_>,
    seed: &SeedUser,
) -> Result<Option<(User, EmailAddress)>> {
    // Check if email already exists
    let existing: Option<(i64,)> = sqlx::query_as(
        "select id from email_addresses where address = $1 and valid_to = 'infinity'",
    )
    .bind(&seed.email)
    .fetch_optional(pool)
    .await
    .context("Failed to check if email exists")?;

    if existing.is_some() {
        return Ok(None);
    }

    let password_hash = hash_password(argon2, &seed.password)?;

    let user = sqlx::query_as::<_, User>(
        "insert into users (password_hash) values ($1) returning id, created_at",
    )
    .bind(&password_hash)
    .fetch_one(pool)
    .await
    .context("Failed to insert user")?;

    let email_address = sqlx::query_as::<_, EmailAddress>(
        "insert into email_addresses (user_id, address)
         values ($1, $2)
         returning id, user_id, address, valid_from, valid_to",
    )
    .bind(user.id)
    .bind(&seed.email)
    .fetch_one(pool)
    .await
    .context("Failed to insert email address")?;

    // Mark email as verified for seed users
    sqlx::query("insert into email_verifications (email_address_id) values ($1)")
        .bind(email_address.id)
        .execute(pool)
        .await
        .context("Failed to verify email")?;

    Ok(Some((user, email_address)))
}

async fn seed_tenant(
    pool: &PgPool,
    seed: &SeedTenant,
    added_by: i64,
) -> Result<Option<(Tenant, TenantDomain)>> {
    // Check if tenant with this name exists
    let existing: Option<(i64,)> = sqlx::query_as("select id from tenants where name = $1")
        .bind(&seed.name)
        .fetch_optional(pool)
        .await
        .context("Failed to check if tenant exists")?;

    if let Some((tenant_id,)) = existing {
        // Check if domain exists for this tenant
        let domain_exists: Option<(i64,)> = sqlx::query_as(
            "select id from tenant_domains
             where tenant_id = $1 and domain = $2 and valid_to = 'infinity'",
        )
        .bind(tenant_id)
        .bind(&seed.domain)
        .fetch_optional(pool)
        .await
        .context("Failed to check if tenant domain exists")?;

        if domain_exists.is_some() {
            return Ok(None);
        }
    }

    // Create tenant
    let tenant = sqlx::query_as::<_, Tenant>(
        "insert into tenants (name) values ($1) returning id, name, created_at",
    )
    .bind(&seed.name)
    .fetch_one(pool)
    .await
    .context("Failed to insert tenant")?;

    // Create domain
    let tenant_domain = sqlx::query_as::<_, TenantDomain>(
        "insert into tenant_domains (tenant_id, domain, added_by)
         values ($1, $2, $3)
         returning id, tenant_id, domain, valid_from, valid_to",
    )
    .bind(tenant.id)
    .bind(&seed.domain)
    .bind(added_by)
    .fetch_one(pool)
    .await
    .context("Failed to insert tenant domain")?;

    Ok(Some((tenant, tenant_domain)))
}

pub async fn seed_all(
    pool: &PgPool,
    argon2: &argon2::Argon2<'_>,
    seeds: &Seeds,
) -> Result<SeedData> {
    let mut users = vec![];
    let mut tenants = vec![];

    // Seed users first
    for seed in &seeds.user {
        if let Some(user_data) = seed_user(pool, argon2, seed).await? {
            users.push(user_data);
        }
    }

    // Use first user as the one who added domains, or default to 1
    let added_by = users.first().map(|(u, _)| u.id).unwrap_or(1);

    // Seed tenants
    for seed in &seeds.tenant {
        if let Some(tenant_data) = seed_tenant(pool, seed, added_by).await? {
            tenants.push(tenant_data);
        }
    }

    Ok(SeedData { users, tenants })
}

pub struct SeedData {
    pub users: Vec<(User, EmailAddress)>,
    pub tenants: Vec<(Tenant, TenantDomain)>,
}
