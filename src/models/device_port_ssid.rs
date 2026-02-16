//! DevicePortSsid model - wireless SSID information per port.

use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DevicePortSsid {
    pub ip: IpNetwork,
    pub port: String,
    pub ssid: Option<String>,
    pub broadcast: Option<bool>,
    pub bssid: Option<String>,
}
