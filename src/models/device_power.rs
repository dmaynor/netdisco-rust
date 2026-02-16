//! DevicePower model - power supply modules on a device.

use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DevicePower {
    pub ip: IpNetwork,
    pub module: i32,
    pub power: Option<i32>,
    pub status: Option<String>,
}
