//! DevicePortPower model - PoE status per port.

use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DevicePortPower {
    pub ip: IpNetwork,
    pub port: String,
    pub module: Option<i32>,
    pub admin: Option<String>,
    pub status: Option<String>,
    pub class: Option<String>,
    pub power: Option<i32>,
}
