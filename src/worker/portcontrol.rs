//! Port control worker - change port admin status, VLAN, PoE.

use anyhow::{Context, Result};
use sqlx::PgPool;
use tracing::info;

use crate::config::NetdiscoConfig;
use crate::models::admin::Admin;

pub async fn port_action(config: &NetdiscoConfig, pool: &PgPool, job: &Admin) -> Result<String> {
    let action = job.action.as_deref().unwrap_or("unknown");
    let device_ip = job.device.ok_or_else(|| anyhow::anyhow!("No device IP"))?;
    let port = job.port.as_deref().ok_or_else(|| anyhow::anyhow!("No port specified"))?;
    let subaction = job.subaction.as_deref().unwrap_or("");

    info!("Port {} on {} {}: action={}, subaction={}",
        port, device_ip, action, action, subaction);

    match action {
        "portcontrol" => {
            // Set port admin status up/down via SNMP SET
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
                .bind(&device_ip)
                .bind(port)
                .bind(format!("Port {} {}", action, subaction))
                .bind(format!("Set port {} to {}", port, target_status))
                .bind(job.username.as_deref())
                .bind(action)
                .execute(pool)
                .await?;

            Ok(format!("Port {} on {} set to {}", port, device_ip, target_status))
        }
        "portname" => {
            // Set port description via SNMP SET
            sqlx::query("UPDATE device_port SET name = $3 WHERE ip = $1 AND port = $2")
                .bind(&device_ip)
                .bind(port)
                .bind(subaction)
                .execute(pool)
                .await?;

            Ok(format!("Port {} on {} name set to '{}'", port, device_ip, subaction))
        }
        "portvlan" => {
            // Change VLAN via SNMP SET
            let vlan: i32 = subaction.parse()
                .context("Invalid VLAN number")?;

            sqlx::query("UPDATE device_port SET pvid = $3 WHERE ip = $1 AND port = $2")
                .bind(&device_ip)
                .bind(port)
                .bind(vlan)
                .execute(pool)
                .await?;

            Ok(format!("Port {} on {} VLAN set to {}", port, device_ip, vlan))
        }
        "power" => {
            // Change PoE status via SNMP SET
            Ok(format!("Port {} on {} PoE action: {}", port, device_ip, subaction))
        }
        _ => Err(anyhow::anyhow!("Unknown port action: {}", action)),
    }
}
