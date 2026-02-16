//! Device discovery worker.
//!
//! Discovers network devices via SNMP, collecting system info, interfaces,
//! VLANs, neighbors, and modules.

use anyhow::{Context, Result};
use ipnetwork::IpNetwork;
use sqlx::PgPool;
use tracing::{info, warn, error, debug};

use crate::config::NetdiscoConfig;
use crate::db;
use crate::snmp::SnmpClient;
use crate::models::device::Device;
use crate::util::permission;

/// Discover a single device by IP address.
pub async fn discover_device(config: &NetdiscoConfig, pool: &PgPool, ip: &IpNetwork) -> Result<String> {
    // Check ACL before proceeding
    if !permission::is_permitted(ip, &config.discover_only, &config.discover_no) {
        return Err(anyhow::anyhow!("Device {} is not permitted by discover ACL", ip));
    }

    info!("Discovering device {}", ip);

    let host = ip.ip().to_string();
    let client = SnmpClient::from_config(config, &host)
        .context("Failed to create SNMP client")?;

    // 1. Get system information
    let sys_info = client.get_system_info()
        .context("Failed to get system info")?;

    info!("  sysName: {:?}", sys_info.name);
    info!("  sysDescr: {:?}", sys_info.description);

    // Determine layers from sysServices
    let layers = sys_info.services.map(|svc| {
        (0..7).map(|i| if svc & (1 << i) != 0 { '1' } else { '0' })
            .collect::<String>()
    });

    // 2. Build device record
    let device = Device {
        ip: *ip,
        creation: None,
        dns: None, // DNS resolution will fill this
        description: sys_info.description,
        uptime: sys_info.uptime,
        contact: sys_info.contact,
        name: sys_info.name,
        location: sys_info.location,
        layers,
        ports: None,
        mac: None,
        serial: None,
        model: None,
        ps1_type: None, ps2_type: None, ps1_status: None, ps2_status: None,
        fan: None, slots: None,
        vendor: None,
        os: None, os_ver: None,
        log: None,
        snmp_ver: Some(config.snmpver as i32),
        snmp_comm: config.community.first().cloned(),
        snmp_class: None,
        vtp_domain: None,
        last_discover: Some(chrono::Local::now().naive_local()),
        last_macsuck: None,
        last_arpnip: None,
        pae_is_enabled: None,
        custom_fields: None,
        tags: None,
    };

    // Store device
    db::upsert_device(pool, &device).await
        .context("Failed to store device")?;

    // 3. Discover interfaces
    let interfaces = match client.get_interfaces() {
        Ok(ifaces) => ifaces,
        Err(e) => {
            warn!("Failed to enumerate interfaces for {}: {}", ip, e);
            Vec::new()
        }
    };
    info!("  Found {} interfaces", interfaces.len());

    for iface in &interfaces {
        let port = crate::models::device_port::DevicePort {
            ip: *ip,
            port: iface.descr.clone(),
            creation: None,
            descr: Some(iface.descr.clone()),
            up: iface.oper_status.map(|s| if s == 1 { "up".to_string() } else { "down".to_string() }),
            up_admin: iface.admin_status.map(|s| if s == 1 { "up".to_string() } else { "down".to_string() }),
            port_type: iface.if_type.clone(),
            duplex: None,
            duplex_admin: None,
            speed: iface.speed.map(|s| s.to_string()),
            name: None,
            mac: None,
            mtu: None,
            stp: None,
            remote_ip: None, remote_port: None, remote_type: None, remote_id: None,
            vlan: None,
            pvid: None,
            lastchange: None,
            ifindex: Some(iface.ifindex),
            is_uplink: None,
            speed_admin: None,
            is_master: None,
            slave_of: None,
            custom_fields: None,
            tags: None,
        };

        if let Err(e) = sqlx::query(
            r#"INSERT INTO device_port (ip, port, descr, up, up_admin, type, speed, ifindex)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               ON CONFLICT (port, ip) DO UPDATE SET
                descr = EXCLUDED.descr,
                up = EXCLUDED.up,
                up_admin = EXCLUDED.up_admin,
                type = EXCLUDED.type,
                speed = EXCLUDED.speed,
                ifindex = EXCLUDED.ifindex"#
        )
            .bind(port.ip)
            .bind(&port.port)
            .bind(&port.descr)
            .bind(&port.up)
            .bind(&port.up_admin)
            .bind(&port.port_type)
            .bind(&port.speed)
            .bind(port.ifindex)
            .execute(pool)
            .await
        {
            error!("Failed to store port {} on {}: {}", port.port, ip, e);
        }
    }

    // 4. Try to discover neighbors (LLDP/CDP)
    if let Err(e) = discover_neighbors(config, pool, ip, &client).await {
        warn!("Failed to discover neighbors for {}: {}", ip, e);
    }

    let msg = format!("Discovered {} with {} interfaces", ip, interfaces.len());
    info!("{}", msg);
    Ok(msg)
}

/// Discover neighbors via LLDP and CDP.
async fn discover_neighbors(
    config: &NetdiscoConfig,
    _pool: &PgPool,
    device_ip: &IpNetwork,
    client: &SnmpClient,
) -> Result<()> {
    if !config.discover_neighbors {
        return Ok(());
    }

    debug!("Discovering neighbors for {}", device_ip);

    // Try LLDP first
    let lldp_names = client.walk(&crate::snmp::oids::LLDP_REM_SYS_NAME).unwrap_or_default();
    let _lldp_ports = client.walk(&crate::snmp::oids::LLDP_REM_PORT_ID).unwrap_or_default();
    let _lldp_addrs = client.walk(&crate::snmp::oids::LLDP_REM_MAN_ADDR).unwrap_or_default();

    for (_oid, name_bytes) in &lldp_names {
        let remote_name = String::from_utf8_lossy(name_bytes).to_string();
        info!("  LLDP neighbor: {}", remote_name);
        // TODO: resolve name to IP/auto-discover
    }

    // Try CDP
    let cdp_devices = client.walk(&crate::snmp::oids::CDP_CACHE_DEVICE_ID).unwrap_or_default();
    for (_oid, device_bytes) in &cdp_devices {
        let remote_device = String::from_utf8_lossy(device_bytes).to_string();
        info!("  CDP neighbor: {}", remote_device);
    }

    Ok(())
}

/// Discover all known devices (scheduled task).
pub async fn discover_all(config: &NetdiscoConfig, pool: &PgPool) -> Result<String> {
    info!("Starting discovery of all devices");
    let devices = db::list_devices(pool, None).await?;
    let total = devices.len();
    let mut success = 0;
    let mut failed = 0;

    for device in &devices {
        match discover_device(config, pool, &device.ip).await {
            Ok(_) => success += 1,
            Err(e) => {
                warn!("Failed to discover {}: {}", device.ip, e);
                failed += 1;
            }
        }
    }

    let msg = format!("DiscoverAll complete: {}/{} succeeded, {} failed", success, total, failed);
    info!("{}", msg);
    Ok(msg)
}
