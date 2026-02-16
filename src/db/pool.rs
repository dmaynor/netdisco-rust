//! Database connection pool management.

use anyhow::{Context, Result};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use tracing::info;

use crate::config::DatabaseConfig;

/// Application database state shared across the application.
#[derive(Debug, Clone)]
pub struct DbPool {
    pub pool: PgPool,
}

impl DbPool {
    /// Create a new database connection pool.
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        let conn_str = config.connection_string();
        // Log only non-sensitive connection details
        info!("Connecting to database '{}' at {}:{}", config.name, config.host, config.port);

        let pool = PgPoolOptions::new()
            .min_connections(2)
            .max_connections(20)
            .idle_timeout(Duration::from_secs(600))
            .max_lifetime(Duration::from_secs(1800))
            .connect(&conn_str)
            .await
            .with_context(|| format!("Failed to connect to database '{}' at {}:{}", config.name, config.host, config.port))?;

        info!("Database connection pool established");
        Ok(Self { pool })
    }

    /// Get a reference to the underlying pool.
    pub fn get_pool(&self) -> &PgPool {
        &self.pool
    }

    /// Test the database connection.
    pub async fn ping(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .context("Database ping failed")?;
        Ok(())
    }
}
