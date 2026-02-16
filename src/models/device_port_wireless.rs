//! DevicePortWireless model - wireless radio info per port.

use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DevicePortWireless {
    pub ip: IpNetwork,
    pub port: String,
    pub channel: Option<i32>,
    pub power: Option<i32>,
}
