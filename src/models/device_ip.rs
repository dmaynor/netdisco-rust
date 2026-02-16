//! DeviceIp model - IP aliases configured on device interfaces.

use chrono::NaiveDateTime;
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// An IP address configured on a device interface.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DeviceIp {
    /// Device primary IP
    pub ip: IpNetwork,
    /// Interface IP alias
    pub alias: IpNetwork,
    /// Subnet in CIDR notation
    pub subnet: Option<IpNetwork>,
    /// Port this IP is configured on
    pub port: Option<String>,
    /// DNS name for this alias
    pub dns: Option<String>,
    /// When first seen
    pub creation: Option<NaiveDateTime>,
}
