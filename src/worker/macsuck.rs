//! MAC address table collection (macsuck) worker.

use anyhow::{Context, Result};
use ipnetwork::IpNetwork;
use sqlx::PgPool;
use tracing::{info, warn, error, debug};

use crate::config::NetdiscoConfig;
use crate::db;
use crate::snmp::SnmpClient;
use crate::models::node::Node;
use crate::util::permission;

/// Collect MAC address table from a single device.
pub async fn macsuck_device(config: &NetdiscoConfig, pool: &PgPool, ip: &IpNetwork) -> Result<String> {
    // Check ACL before proceeding
    if !permission::is_permitted(ip, &config.macsuck_only, &config.macsuck_no) {
        return Err(anyhow::anyhow!("Device {} is not permitted by macsuck ACL", ip));
    }

    info!("Macsucking device {}", ip);

    let host = ip.ip().to_string();
    let client = SnmpClient::from_config(config, &host)?;

    // Get MAC address table via SNMP
    let mac_entries = client.get_mac_table()
        .context("Failed to get MAC table")?;

    info!("  Found {} MAC entries", mac_entries.len());

    let mut stored = 0;
    for entry in &mac_entries {
        // Store each MAC entry
        let node = Node {
            mac: entry.mac.clone(),
            switch: *ip,
            port: format!("bridge-port-{}", entry.bridge_port),
            vlan: entry.vlan.map(|v| v.to_string()),
            active: Some(true),
            oui: Some(Node::extract_oui(&entry.mac)),
            time_first: None,
            time_recent: None,
            time_last: None,
        };

        if let Err(e) = db::upsert_node(pool, &node).await {
            debug!("Failed to store MAC {}: {}", entry.mac, e);
        } else {
            stored += 1;
        }
    }

    // Mark old entries as inactive
    if let Err(e) = sqlx::query(
        "UPDATE node SET active = false WHERE switch = $1 AND time_last < NOW() - interval '5 minutes' AND active = true"
    )
        .bind(ip)
        .execute(pool)
        .await
    {
        error!("Failed to deactivate old nodes for {}: {}", ip, e);
    }

    // Update last_macsuck timestamp
    if let Err(e) = sqlx::query("UPDATE device SET last_macsuck = NOW() WHERE ip = $1")
        .bind(ip)
        .execute(pool)
        .await
    {
        error!("Failed to update last_macsuck for {}: {}", ip, e);
    }

    let msg = format!("Macsuck {}: stored {} of {} MACs", ip, stored, mac_entries.len());
    info!("{}", msg);
    Ok(msg)
}

/// Walk all devices for MAC address collection (scheduled).
pub async fn macwalk(config: &NetdiscoConfig, pool: &PgPool) -> Result<String> {
    info!("Starting MAC walk of all devices");
    let devices = db::list_devices(pool, None).await?;

    // Only macsuck layer 2 devices
    let l2_devices: Vec<_> = devices.iter()
        .filter(|d| d.has_layer(2))
        .collect();

    let total = l2_devices.len();
    let mut success = 0;

    for device in l2_devices {
        match macsuck_device(config, pool, &device.ip).await {
            Ok(_) => success += 1,
            Err(e) => warn!("Macsuck failed for {}: {}", device.ip, e),
        }
    }

    let msg = format!("Macwalk complete: {}/{} devices", success, total);
    info!("{}", msg);
    Ok(msg)
}
