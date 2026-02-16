//! ARP/NDP table collection (arpnip) worker.

use anyhow::{Context, Result};
use ipnetwork::IpNetwork;
use sqlx::PgPool;
use tracing::{info, warn, error, debug};

use crate::config::NetdiscoConfig;
use crate::db;
use crate::snmp::SnmpClient;
use crate::util::permission;

/// Collect ARP table from a single device.
pub async fn arpnip_device(config: &NetdiscoConfig, pool: &PgPool, ip: &IpNetwork) -> Result<String> {
    // Check ACL before proceeding
    if !permission::is_permitted(ip, &config.arpnip_only, &config.arpnip_no) {
        return Err(anyhow::anyhow!("Device {} is not permitted by arpnip ACL", ip));
    }

    info!("Arpnipping device {}", ip);

    let host = ip.ip().to_string();
    let client = SnmpClient::from_config(config, &host)?;

    let arp_entries = client.get_arp_table()
        .context("Failed to get ARP table")?;

    info!("  Found {} ARP entries", arp_entries.len());

    let mut stored = 0;
    for entry in &arp_entries {
        if let Ok(entry_ip) = entry.ip.parse::<std::net::IpAddr>() {
            let ip_network = IpNetwork::from(entry_ip);
            if let Err(e) = db::upsert_node_ip(pool, &entry.mac, &ip_network).await {
                debug!("Failed to store ARP {}->{}: {}", entry.mac, entry.ip, e);
            } else {
                stored += 1;
            }
        }
    }

    // Mark old entries as inactive
    if let Err(e) = sqlx::query(
        "UPDATE node_ip SET active = false WHERE time_last < NOW() - interval '5 minutes' AND active = true"
    )
        .execute(pool)
        .await
    {
        error!("Failed to deactivate old ARP entries: {}", e);
    }

    // Update last_arpnip timestamp
    if let Err(e) = sqlx::query("UPDATE device SET last_arpnip = NOW() WHERE ip = $1")
        .bind(ip)
        .execute(pool)
        .await
    {
        error!("Failed to update last_arpnip for {}: {}", ip, e);
    }

    let msg = format!("Arpnip {}: stored {} of {} entries", ip, stored, arp_entries.len());
    info!("{}", msg);
    Ok(msg)
}

/// Walk all devices for ARP table collection (scheduled).
pub async fn arpwalk(config: &NetdiscoConfig, pool: &PgPool) -> Result<String> {
    info!("Starting ARP walk of all devices");
    let devices = db::list_devices(pool, None).await?;

    // Only arpnip layer 3 devices
    let l3_devices: Vec<_> = devices.iter()
        .filter(|d| d.has_layer(3))
        .collect();

    let total = l3_devices.len();
    let mut success = 0;

    for device in l3_devices {
        match arpnip_device(config, pool, &device.ip).await {
            Ok(_) => success += 1,
            Err(e) => warn!("Arpnip failed for {}: {}", device.ip, e),
        }
    }

    let msg = format!("Arpwalk complete: {}/{} devices", success, total);
    info!("{}", msg);
    Ok(msg)
}
