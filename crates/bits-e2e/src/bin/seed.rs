use anyhow::{Context, Result};
use bits_e2e::seeds;

#[tokio::main]
async fn main() -> Result<()> {
    let config = bits_app::load_config()?;
    let state = bits_app::AppState::new(config).await?;

    let seeds_path = std::env::current_dir()?.join("seeds.toml");
    let seeds = seeds::load_seeds(&seeds_path)
        .context("Failed to load seeds.toml (expected at workspace root)")?;

    let result = seeds::seed_all(&state.db, &state.password_service, &seeds)
        .await
        .context("Failed to seed database")?;

    eprintln!(
        "Seeded {} user(s) and {} tenant(s)",
        result.users.len(),
        result.tenants.len()
    );

    Ok(())
}
