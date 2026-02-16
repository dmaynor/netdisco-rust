//! Device model - represents a network device (switch, router, AP).
//!
//! Maps to the `device` table in PostgreSQL.

use chrono::NaiveDateTime;
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// A network device discovered and managed by Netdisco.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Device {
    /// Primary key - device IP address
    pub ip: IpNetwork,
    /// When this device was first seen
    pub creation: Option<NaiveDateTime>,
    /// DNS hostname
    pub dns: Option<String>,
    /// sysDescr from SNMP
    pub description: Option<String>,
    /// sysUpTime from SNMP (in hundredths of a second)
    pub uptime: Option<i64>,
    /// sysContact from SNMP
    pub contact: Option<String>,
    /// sysName from SNMP
    pub name: Option<String>,
    /// sysLocation from SNMP
    pub location: Option<String>,
    /// OSI layer capabilities (7 chars, e.g. "0000011")
    pub layers: Option<String>,
    /// Number of interfaces
    pub ports: Option<i32>,
    /// Device MAC address
    pub mac: Option<String>,
    /// Serial number
    pub serial: Option<String>,
    /// Hardware model
    pub model: Option<String>,
    /// Power supply 1 type
    pub ps1_type: Option<String>,
    /// Power supply 2 type
    pub ps2_type: Option<String>,
    /// Power supply 1 status
    pub ps1_status: Option<String>,
    /// Power supply 2 status
    pub ps2_status: Option<String>,
    /// Fan status
    pub fan: Option<String>,
    /// Number of slots
    pub slots: Option<i32>,
    /// Hardware vendor
    pub vendor: Option<String>,
    /// Operating system
    pub os: Option<String>,
    /// OS version
    pub os_ver: Option<String>,
    /// Log info
    pub log: Option<String>,
    /// SNMP version used (1, 2, or 3)
    pub snmp_ver: Option<i32>,
    /// SNMP community string
    pub snmp_comm: Option<String>,
    /// SNMP::Info class used
    pub snmp_class: Option<String>,
    /// VTP domain
    pub vtp_domain: Option<String>,
    /// Last successful discover timestamp
    pub last_discover: Option<NaiveDateTime>,
    /// Last successful macsuck timestamp
    pub last_macsuck: Option<NaiveDateTime>,
    /// Last successful arpnip timestamp
    pub last_arpnip: Option<NaiveDateTime>,
    /// Whether 802.1X (PAE) is enabled
    pub pae_is_enabled: Option<bool>,
    /// Custom fields (JSONB)
    pub custom_fields: Option<serde_json::Value>,
    /// Tags array
    pub tags: Option<Vec<String>>,
}

impl Device {
    /// Check if device supports a specific OSI layer (1-7).
    pub fn has_layer(&self, layer: u8) -> bool {
        if layer == 0 || layer > 7 {
            return false;
        }
        match &self.layers {
            Some(layers) => {
                let idx = (7 - layer) as usize;
                layers.chars().nth(idx).map_or(false, |c| c == '1')
            }
            None => false,
        }
    }

    /// Returns true if this device is a Layer 2 switch.
    pub fn is_switch(&self) -> bool {
        self.has_layer(2)
    }

    /// Returns true if this device is a Layer 3 router.
    pub fn is_router(&self) -> bool {
        self.has_layer(3)
    }

    /// Display name - prefers DNS, then sysName, then IP.
    pub fn display_name(&self) -> String {
        self.dns
            .as_deref()
            .filter(|s| !s.is_empty())
            .or(self.name.as_deref().filter(|s| !s.is_empty()))
            .unwrap_or(&self.ip.to_string())
            .to_string()
    }
}

/// For creating a new device record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewDevice {
    pub ip: IpNetwork,
    pub dns: Option<String>,
    pub description: Option<String>,
    pub uptime: Option<i64>,
    pub contact: Option<String>,
    pub name: Option<String>,
    pub location: Option<String>,
    pub layers: Option<String>,
    pub ports: Option<i32>,
    pub mac: Option<String>,
    pub serial: Option<String>,
    pub model: Option<String>,
    pub vendor: Option<String>,
    pub os: Option<String>,
    pub os_ver: Option<String>,
    pub snmp_ver: Option<i32>,
    pub snmp_comm: Option<String>,
    pub snmp_class: Option<String>,
}
