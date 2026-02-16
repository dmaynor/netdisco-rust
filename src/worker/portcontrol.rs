//! Port control worker - change port admin status, VLAN, PoE.
//!
//! NOTE: SNMP SET operations are not yet implemented. These actions currently
//! only update the local database. The physical device is NOT modified.

use anyhow::{Context, Result};
use sqlx::PgPool;
use tracing::{info, warn};

use crate::config::NetdiscoConfig;
use crate::models::admin::Admin;

pub async fn port_action(_config: &NetdiscoConfig, pool: &PgPool, job: &Admin) -> Result<String> {
    let action = job.action.as_deref().unwrap_or("unknown");
    let device_ip = job.device.ok_or_else(|| anyhow::anyhow!("No device IP"))?;
    let port = job.port.as_deref().ok_or_else(|| anyhow::anyhow!("No port specified"))?;
    let subaction = job.subaction.as_deref().unwrap_or("");

    info!("Port {} on {} {}: action={}, subaction={}",
        port, device_ip, action, action, subaction);

    warn!("SNMP SET not yet implemented - only updating local database, device {} is NOT modified", device_ip);

    match action {
        "portcontrol" => {
            let target_status = match subaction {
                "up" | "enable" => "up",
                "down" | "disable" => "down",
                _ => return Err(anyhow::anyhow!("Invalid port control subaction: {}", subaction)),
            };

            // Log the port change
            sqlx::query(
                r#"INSERT INTO device_port_log (ip, port, reason, log, username, action)
                   VALUES ($1, $2, $3, $4, $5, $6)"#
            )
                .bind(device_ip)
                .bind(port)
                .bind(format!("Port {} {}", action, subaction))
                .bind(format!("Set port {} to {} (DB only, SNMP SET not implemented)", port, target_status))
                .bind(job.username.as_deref())
                .bind(action)
                .execute(pool)
                .await?;

            Ok(format!("Port {} on {} set to {} (DB only)", port, device_ip, target_status))
        }
        "portname" => {
            sqlx::query("UPDATE device_port SET name = $3 WHERE ip = $1 AND port = $2")
                .bind(device_ip)
                .bind(port)
                .bind(subaction)
                .execute(pool)
                .await?;

            Ok(format!("Port {} on {} name set to '{}' (DB only)", port, device_ip, subaction))
        }
        "portvlan" => {
            let vlan: i32 = subaction.parse()
                .context("Invalid VLAN number")?;

            sqlx::query("UPDATE device_port SET pvid = $3 WHERE ip = $1 AND port = $2")
                .bind(device_ip)
                .bind(port)
                .bind(vlan)
                .execute(pool)
                .await?;

            Ok(format!("Port {} on {} VLAN set to {} (DB only)", port, device_ip, vlan))
        }
        "power" => {
            warn!("PoE control not implemented for port {} on {}", port, device_ip);
            Err(anyhow::anyhow!("PoE control is not yet implemented"))
        }
        _ => Err(anyhow::anyhow!("Unknown port action: {}", action)),
    }
}
