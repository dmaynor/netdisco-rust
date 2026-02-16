//! DevicePort model - represents a physical or logical port on a device.

use chrono::NaiveDateTime;
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// A port (interface) on a network device.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DevicePort {
    /// Device IP address
    pub ip: IpNetwork,
    /// Port identifier (e.g., "GigabitEthernet0/1")
    pub port: String,
    /// When this port was first seen
    pub creation: Option<NaiveDateTime>,
    /// Interface description (ifDescr)
    pub descr: Option<String>,
    /// Operational status (up/down)
    pub up: Option<String>,
    /// Administrative status (up/down)
    pub up_admin: Option<String>,
    /// Interface type (e.g., "ethernetCsmacd")
    #[sqlx(rename = "type")]
    pub port_type: Option<String>,
    /// Running duplex (full/half)
    pub duplex: Option<String>,
    /// Admin duplex setting
    pub duplex_admin: Option<String>,
    /// Running speed
    pub speed: Option<String>,
    /// Friendly port name (ifAlias)
    pub name: Option<String>,
    /// Port MAC address
    pub mac: Option<String>,
    /// Maximum transmission unit
    pub mtu: Option<i32>,
    /// Spanning tree state
    pub stp: Option<String>,
    /// Remote device IP (CDP/LLDP neighbor)
    pub remote_ip: Option<IpNetwork>,
    /// Remote device port
    pub remote_port: Option<String>,
    /// Remote device type
    pub remote_type: Option<String>,
    /// Remote device ID
    pub remote_id: Option<String>,
    /// VLAN name
    pub vlan: Option<String>,
    /// Native VLAN ID
    pub pvid: Option<i32>,
    /// Last change counter (sysUpTime ticks)
    pub lastchange: Option<i64>,
    /// ifIndex
    pub ifindex: Option<i32>,
    /// Whether this is an uplink
    pub is_uplink: Option<bool>,
    /// Configured speed
    pub speed_admin: Option<String>,
    /// PoE enabled
    pub is_master: Option<bool>,
    /// Slave of (LAG member)
    pub slave_of: Option<String>,
    /// Custom fields (JSONB)
    pub custom_fields: Option<serde_json::Value>,
    /// Tags array
    pub tags: Option<Vec<String>>,
}

/// For creating a new device port record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewDevicePort {
    pub ip: IpNetwork,
    pub port: String,
    pub descr: Option<String>,
    pub up: Option<String>,
    pub up_admin: Option<String>,
    pub port_type: Option<String>,
    pub duplex: Option<String>,
    pub speed: Option<String>,
    pub name: Option<String>,
    pub mac: Option<String>,
    pub mtu: Option<i32>,
    pub pvid: Option<i32>,
}
