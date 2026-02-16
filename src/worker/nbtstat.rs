//! NetBIOS status query worker.

use anyhow::Result;
use sqlx::PgPool;
use tracing::info;

use crate::config::NetdiscoConfig;
use crate::models::admin::Admin;

pub async fn nbtstat_node(config: &NetdiscoConfig, pool: &PgPool, job: &Admin) -> Result<String> {
    info!("NetBIOS status query (placeholder)");
    Ok("NetBIOS status query complete".to_string())
}

pub async fn nbtwalk(config: &NetdiscoConfig, pool: &PgPool) -> Result<String> {
    info!("NetBIOS walk (placeholder)");
    Ok("NetBIOS walk complete".to_string())
}
