//! Data expiration worker.
//!
//! Removes old device, node, and job records based on configured thresholds.

use anyhow::Result;
use sqlx::PgPool;
use tracing::info;

use crate::config::NetdiscoConfig;

pub async fn expire(config: &NetdiscoConfig, pool: &PgPool) -> Result<String> {
    info!("Running data expiration");

    let mut messages = Vec::new();

    // Expire old devices
    let device_interval = format!("{} days", config.expire_devices);
    let result = sqlx::query(
        "DELETE FROM device WHERE last_discover < NOW() - $1::interval"
    )
        .bind(&device_interval)
        .execute(pool)
        .await?;
    messages.push(format!("Expired {} devices older than {} days", result.rows_affected(), config.expire_devices));

    // Expire old archived nodes
    let node_archive_interval = format!("{} days", config.expire_nodes_archive);
    let result = sqlx::query(
        "DELETE FROM node WHERE active = false AND time_last < NOW() - $1::interval"
    )
        .bind(&node_archive_interval)
        .execute(pool)
        .await?;
    messages.push(format!("Expired {} archived nodes older than {} days", result.rows_affected(), config.expire_nodes_archive));

    // Expire old node_ip records
    let node_interval = format!("{} days", config.expire_nodes);
    let result = sqlx::query(
        "DELETE FROM node_ip WHERE active = false AND time_last < NOW() - $1::interval"
    )
        .bind(&node_interval)
        .execute(pool)
        .await?;
    messages.push(format!("Expired {} node_ip records older than {} days", result.rows_affected(), config.expire_nodes));

    // Expire old jobs
    let job_interval = format!("{} days", config.expire_jobs);
    let result = sqlx::query(
        "DELETE FROM admin WHERE finished IS NOT NULL AND finished < NOW() - $1::interval"
    )
        .bind(&job_interval)
        .execute(pool)
        .await?;
    messages.push(format!("Expired {} jobs older than {} days", result.rows_affected(), config.expire_jobs));

    // Expire old user log entries
    let userlog_interval = format!("{} days", config.expire_userlog);
    let result = sqlx::query(
        "DELETE FROM user_log WHERE creation < NOW() - $1::interval"
    )
        .bind(&userlog_interval)
        .execute(pool)
        .await?;
    messages.push(format!("Expired {} user_log entries older than {} days", result.rows_affected(), config.expire_userlog));

    let msg = messages.join("; ");
    info!("{}", msg);
    Ok(msg)
}
