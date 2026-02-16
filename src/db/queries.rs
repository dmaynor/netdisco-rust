//! Database queries for all Netdisco models.
//!
//! Provides CRUD operations matching the Perl DBIx::Class layer.

use anyhow::Result;
use ipnetwork::IpNetwork;
use sqlx::PgPool;
use crate::models::*;

// ==================== Device Queries ====================

/// Find a device by IP address.
pub async fn find_device(pool: &PgPool, ip: &IpNetwork) -> Result<Option<Device>> {
    let device = sqlx::query_as::<_, Device>("SELECT * FROM device WHERE ip = $1")
        .bind(ip)
        .fetch_optional(pool)
        .await?;
    Ok(device)
}

/// List all devices, optionally filtered.
pub async fn list_devices(pool: &PgPool, limit: Option<i64>) -> Result<Vec<Device>> {
    let devices = sqlx::query_as::<_, Device>(
        "SELECT * FROM device ORDER BY dns, ip LIMIT $1"
    )
        .bind(limit.unwrap_or(1000))
        .fetch_all(pool)
        .await?;
    Ok(devices)
}

/// Search devices by name, DNS, IP, location, etc.
pub async fn search_devices(pool: &PgPool, query: &str) -> Result<Vec<Device>> {
    let pattern = format!("%{}%", query);
    let devices = sqlx::query_as::<_, Device>(
        r#"SELECT * FROM device
           WHERE dns ILIKE $1
              OR name ILIKE $1
              OR location ILIKE $1
              OR host(ip)::text ILIKE $1
              OR model ILIKE $1
              OR vendor ILIKE $1
           ORDER BY dns, ip
           LIMIT 100"#
    )
        .bind(&pattern)
        .fetch_all(pool)
        .await?;
    Ok(devices)
}

/// Insert or update a device record.
pub async fn upsert_device(pool: &PgPool, device: &Device) -> Result<()> {
    sqlx::query(
        r#"INSERT INTO device (ip, dns, description, uptime, contact, name, location,
            layers, ports, mac, serial, model, vendor, os, os_ver, snmp_ver,
            snmp_comm, snmp_class, vtp_domain, last_discover)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14,
                   $15, $16, $17, $18, $19, $20)
           ON CONFLICT (ip) DO UPDATE SET
            dns = EXCLUDED.dns,
            description = EXCLUDED.description,
            uptime = EXCLUDED.uptime,
            contact = EXCLUDED.contact,
            name = EXCLUDED.name,
            location = EXCLUDED.location,
            layers = EXCLUDED.layers,
            ports = EXCLUDED.ports,
            mac = EXCLUDED.mac,
            serial = EXCLUDED.serial,
            model = EXCLUDED.model,
            vendor = EXCLUDED.vendor,
            os = EXCLUDED.os,
            os_ver = EXCLUDED.os_ver,
            snmp_ver = EXCLUDED.snmp_ver,
            snmp_comm = EXCLUDED.snmp_comm,
            snmp_class = EXCLUDED.snmp_class,
            vtp_domain = EXCLUDED.vtp_domain,
            last_discover = EXCLUDED.last_discover"#
    )
        .bind(device.ip)
        .bind(&device.dns)
        .bind(&device.description)
        .bind(device.uptime)
        .bind(&device.contact)
        .bind(&device.name)
        .bind(&device.location)
        .bind(&device.layers)
        .bind(device.ports)
        .bind(&device.mac)
        .bind(&device.serial)
        .bind(&device.model)
        .bind(&device.vendor)
        .bind(&device.os)
        .bind(&device.os_ver)
        .bind(device.snmp_ver)
        .bind(&device.snmp_comm)
        .bind(&device.snmp_class)
        .bind(&device.vtp_domain)
        .bind(device.last_discover)
        .execute(pool)
        .await?;
    Ok(())
}

/// Delete a device and all related records.
pub async fn delete_device(pool: &PgPool, ip: &IpNetwork) -> Result<()> {
    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM node WHERE switch = $1").bind(ip).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM device_port_vlan WHERE ip = $1").bind(ip).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM device_port_power WHERE ip = $1").bind(ip).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM device_port_ssid WHERE ip = $1").bind(ip).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM device_port_wireless WHERE ip = $1").bind(ip).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM device_port_log WHERE ip = $1").bind(ip).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM device_port WHERE ip = $1").bind(ip).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM device_module WHERE ip = $1").bind(ip).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM device_ip WHERE ip = $1").bind(ip).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM device_vlan WHERE ip = $1").bind(ip).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM device_power WHERE ip = $1").bind(ip).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM device_skip WHERE device = $1").bind(ip).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM admin WHERE device = $1").bind(ip).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM device WHERE ip = $1").bind(ip).execute(&mut *tx).await?;

    tx.commit().await?;
    Ok(())
}

// ==================== Node Queries ====================

/// Find nodes (MAC addresses) on a specific switch port.
pub async fn find_nodes_on_port(
    pool: &PgPool,
    switch: &IpNetwork,
    port: &str,
    active_only: bool,
) -> Result<Vec<Node>> {
    let query = if active_only {
        "SELECT * FROM node WHERE switch = $1 AND port = $2 AND active = true ORDER BY time_last DESC"
    } else {
        "SELECT * FROM node WHERE switch = $1 AND port = $2 ORDER BY time_last DESC"
    };
    let nodes = sqlx::query_as::<_, Node>(query)
        .bind(switch)
        .bind(port)
        .fetch_all(pool)
        .await?;
    Ok(nodes)
}

/// Search for a node by MAC address.
pub async fn find_node_by_mac(pool: &PgPool, mac: &str) -> Result<Vec<Node>> {
    let nodes = sqlx::query_as::<_, Node>(
        "SELECT * FROM node WHERE mac = $1::macaddr ORDER BY active DESC, time_last DESC"
    )
        .bind(mac)
        .fetch_all(pool)
        .await?;
    Ok(nodes)
}

/// Search for a node by IP address (via node_ip join).
pub async fn find_node_by_ip(pool: &PgPool, ip: &IpNetwork) -> Result<Vec<NodeIp>> {
    let nodes = sqlx::query_as::<_, NodeIp>(
        "SELECT * FROM node_ip WHERE ip = $1 ORDER BY active DESC, time_last DESC"
    )
        .bind(ip)
        .fetch_all(pool)
        .await?;
    Ok(nodes)
}

/// Upsert a node (MAC on switch port).
pub async fn upsert_node(pool: &PgPool, node: &Node) -> Result<()> {
    sqlx::query(
        r#"INSERT INTO node (mac, switch, port, vlan, active, oui, time_first, time_recent, time_last)
           VALUES ($1::macaddr, $2, $3, $4, $5, $6, NOW(), NOW(), NOW())
           ON CONFLICT (mac, switch, port, vlan) DO UPDATE SET
            active = EXCLUDED.active,
            time_recent = NOW(),
            time_last = NOW()"#
    )
        .bind(&node.mac)
        .bind(node.switch)
        .bind(&node.port)
        .bind(&node.vlan)
        .bind(node.active)
        .bind(&node.oui)
        .execute(pool)
        .await?;
    Ok(())
}

/// Upsert a node_ip (MAC to IP mapping from ARP/NDP).
pub async fn upsert_node_ip(pool: &PgPool, mac: &str, ip: &IpNetwork) -> Result<()> {
    sqlx::query(
        r#"INSERT INTO node_ip (mac, ip, active, time_first, time_last)
           VALUES ($1::macaddr, $2, true, NOW(), NOW())
           ON CONFLICT (mac, ip) DO UPDATE SET
            active = true,
            time_last = NOW()"#
    )
        .bind(mac)
        .bind(ip)
        .execute(pool)
        .await?;
    Ok(())
}

// ==================== Admin/Job Queue Queries ====================

/// Add a job to the admin queue.
pub async fn enqueue_job(
    pool: &PgPool,
    action: &str,
    device: Option<&IpNetwork>,
    port: Option<&str>,
    username: Option<&str>,
) -> Result<i32> {
    let row = sqlx::query_scalar::<_, i32>(
        r#"INSERT INTO admin (action, device, port, username, status, entered)
           VALUES ($1, $2, $3, $4, 'queued', NOW())
           RETURNING job"#
    )
        .bind(action)
        .bind(device)
        .bind(port)
        .bind(username)
        .fetch_one(pool)
        .await?;
    Ok(row)
}

/// Fetch the next queued job for processing.
pub async fn dequeue_job(pool: &PgPool) -> Result<Option<Admin>> {
    let job = sqlx::query_as::<_, Admin>(
        r#"UPDATE admin SET status = 'running', started = NOW()
           WHERE job = (
               SELECT job FROM admin
               WHERE status = 'queued'
               ORDER BY entered ASC
               LIMIT 1
               FOR UPDATE SKIP LOCKED
           )
           RETURNING *"#
    )
        .fetch_optional(pool)
        .await?;
    Ok(job)
}

/// Mark a job as completed.
pub async fn complete_job(pool: &PgPool, job_id: i32, status: &str, log: &str) -> Result<()> {
    sqlx::query(
        "UPDATE admin SET status = $2, log = $3, finished = NOW() WHERE job = $1"
    )
        .bind(job_id)
        .bind(status)
        .bind(log)
        .execute(pool)
        .await?;
    Ok(())
}

/// Get pending/recent jobs.
pub async fn list_jobs(pool: &PgPool, limit: i64) -> Result<Vec<Admin>> {
    let jobs = sqlx::query_as::<_, Admin>(
        "SELECT * FROM admin ORDER BY entered DESC LIMIT $1"
    )
        .bind(limit)
        .fetch_all(pool)
        .await?;
    Ok(jobs)
}

// ==================== User Queries ====================

/// Find a user by username.
pub async fn find_user(pool: &PgPool, username: &str) -> Result<Option<user::User>> {
    let user = sqlx::query_as::<_, user::User>(
        "SELECT * FROM users WHERE username = $1"
    )
        .bind(username)
        .fetch_optional(pool)
        .await?;
    Ok(user)
}

/// Create a new user.
pub async fn create_user(
    pool: &PgPool,
    username: &str,
    password_hash: &str,
    admin: bool,
) -> Result<()> {
    sqlx::query(
        r#"INSERT INTO users (username, password, admin, creation)
           VALUES ($1, $2, $3, NOW())
           ON CONFLICT (username) DO UPDATE SET password = $2, admin = $3"#
    )
        .bind(username)
        .bind(password_hash)
        .bind(admin)
        .execute(pool)
        .await?;
    Ok(())
}

// ==================== Device Port Queries ====================

/// Get all ports for a device.
pub async fn get_device_ports(pool: &PgPool, ip: &IpNetwork) -> Result<Vec<DevicePort>> {
    let ports = sqlx::query_as::<_, DevicePort>(
        "SELECT * FROM device_port WHERE ip = $1 ORDER BY port"
    )
        .bind(ip)
        .fetch_all(pool)
        .await?;
    Ok(ports)
}

/// Get VLANs for a device.
pub async fn get_device_vlans(pool: &PgPool, ip: &IpNetwork) -> Result<Vec<DeviceVlan>> {
    let vlans = sqlx::query_as::<_, DeviceVlan>(
        "SELECT * FROM device_vlan WHERE ip = $1 ORDER BY vlan"
    )
        .bind(ip)
        .fetch_all(pool)
        .await?;
    Ok(vlans)
}

/// Get modules for a device.
pub async fn get_device_modules(pool: &PgPool, ip: &IpNetwork) -> Result<Vec<DeviceModule>> {
    let modules = sqlx::query_as::<_, DeviceModule>(
        "SELECT * FROM device_module WHERE ip = $1 ORDER BY index"
    )
        .bind(ip)
        .fetch_all(pool)
        .await?;
    Ok(modules)
}

/// Get IP aliases for a device.
pub async fn get_device_ips(pool: &PgPool, ip: &IpNetwork) -> Result<Vec<device_ip::DeviceIp>> {
    let ips = sqlx::query_as::<_, device_ip::DeviceIp>(
        "SELECT * FROM device_ip WHERE ip = $1 ORDER BY alias"
    )
        .bind(ip)
        .fetch_all(pool)
        .await?;
    Ok(ips)
}

// ==================== Statistics Queries ====================

/// Get device count.
pub async fn device_count(pool: &PgPool) -> Result<i64> {
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM device")
        .fetch_one(pool)
        .await?;
    Ok(count)
}

/// Get node (MAC) count.
pub async fn node_count(pool: &PgPool, active_only: bool) -> Result<i64> {
    let query = if active_only {
        "SELECT COUNT(DISTINCT mac) FROM node WHERE active = true"
    } else {
        "SELECT COUNT(DISTINCT mac) FROM node"
    };
    let count = sqlx::query_scalar::<_, i64>(query)
        .fetch_one(pool)
        .await?;
    Ok(count)
}

/// Get port count.
pub async fn port_count(pool: &PgPool) -> Result<i64> {
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM device_port")
        .fetch_one(pool)
        .await?;
    Ok(count)
}

// ==================== OUI Queries ====================

/// Look up a vendor by OUI prefix.
pub async fn find_oui(pool: &PgPool, oui: &str) -> Result<Option<oui::Oui>> {
    let result = sqlx::query_as::<_, oui::Oui>(
        "SELECT * FROM oui WHERE oui = $1"
    )
        .bind(oui)
        .fetch_optional(pool)
        .await?;
    Ok(result)
}
