//! DeviceSkip model - tracks devices that should be skipped for certain actions.

use chrono::NaiveDateTime;
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DeviceSkip {
    pub backend: Option<String>,
    pub device: IpNetwork,
    pub actionset: Option<Vec<String>>,
    pub deferrals: Option<i32>,
    pub last_defer: Option<NaiveDateTime>,
}
