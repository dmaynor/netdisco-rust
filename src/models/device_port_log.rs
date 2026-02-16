//! DevicePortLog model - history of port status changes and admin actions.

use chrono::NaiveDateTime;
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DevicePortLog {
    pub id: Option<i32>,
    pub ip: Option<IpNetwork>,
    pub port: Option<String>,
    pub reason: Option<String>,
    pub log: Option<String>,
    pub username: Option<String>,
    pub userip: Option<IpNetwork>,
    pub action: Option<String>,
    pub creation: Option<NaiveDateTime>,
}
