//! DevicePortVlan model - VLAN membership per port.

use chrono::NaiveDateTime;
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DevicePortVlan {
    pub ip: IpNetwork,
    pub port: String,
    pub vlan: i32,
    pub native: bool,
    pub creation: Option<NaiveDateTime>,
    pub last_discover: Option<NaiveDateTime>,
    pub vlantype: Option<String>,
}
