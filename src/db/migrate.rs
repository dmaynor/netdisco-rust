//! Database migration system.
//!
//! Handles schema creation and upgrades for the Netdisco database.

use anyhow::{Context, Result};
use sqlx::PgPool;
use tracing::info;

/// Run all pending database migrations.
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    info!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .context("Failed to run database migrations")?;
    info!("Database migrations complete");
    Ok(())
}

/// Check current schema version.
pub async fn schema_version(pool: &PgPool) -> Result<i64> {
    let version = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(MAX(version), 0) FROM _sqlx_migrations"
    )
        .fetch_one(pool)
        .await
        .unwrap_or(0);
    Ok(version)
}
