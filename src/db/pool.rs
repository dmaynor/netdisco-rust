//! Database connection pool management.

use anyhow::{Context, Result};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::sync::Arc;
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
        info!("Connecting to database: {}", config.name);

        let pool = PgPoolOptions::new()
            .max_connections(20)
            .connect(&conn_str)
            .await
            .with_context(|| format!("Failed to connect to database at {}", conn_str))?;

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
