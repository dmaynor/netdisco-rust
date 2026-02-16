//! DeviceVlan model - VLANs known to a device.

use chrono::NaiveDateTime;
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DeviceVlan {
    pub ip: IpNetwork,
    pub vlan: i32,
    pub description: Option<String>,
    pub creation: Option<NaiveDateTime>,
    pub last_discover: Option<NaiveDateTime>,
}
