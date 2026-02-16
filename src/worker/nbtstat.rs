//! NetBIOS status query worker.

use anyhow::Result;
use sqlx::PgPool;
use tracing::warn;

use crate::config::NetdiscoConfig;
use crate::models::admin::Admin;

pub async fn nbtstat_node(_config: &NetdiscoConfig, _pool: &PgPool, _job: &Admin) -> Result<String> {
    warn!("NetBIOS status query not yet implemented");
    Err(anyhow::anyhow!("nbtstat is not yet implemented"))
}

pub async fn nbtwalk(_config: &NetdiscoConfig, _pool: &PgPool) -> Result<String> {
    warn!("NetBIOS walk not yet implemented");
    Err(anyhow::anyhow!("nbtwalk is not yet implemented"))
}
