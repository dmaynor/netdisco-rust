//! DeviceBrowser model - SNMP OID walk data stored for a device.

use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DeviceBrowser {
    pub ip: IpNetwork,
    pub oid: String,
    pub oid_parts: Option<serde_json::Value>,
}
