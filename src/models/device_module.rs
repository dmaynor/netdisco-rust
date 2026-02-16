//! DeviceModule model - hardware modules (line cards, chassis, etc.).

use chrono::NaiveDateTime;
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DeviceModule {
    pub ip: IpNetwork,
    pub index: i32,
    pub description: Option<String>,
    #[sqlx(rename = "type")]
    pub module_type: Option<String>,
    pub parent: Option<i32>,
    pub name: Option<String>,
    pub class: Option<String>,
    pub pos: Option<i32>,
    pub hw_ver: Option<String>,
    pub fw_ver: Option<String>,
    pub sw_ver: Option<String>,
    pub serial: Option<String>,
    pub model: Option<String>,
    pub fru: Option<bool>,
    pub creation: Option<NaiveDateTime>,
    pub last_discover: Option<NaiveDateTime>,
}
