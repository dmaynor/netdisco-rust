//! Process model - coordination between backend workers.

use chrono::NaiveDateTime;
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Process {
    pub controller: i32,
    pub device: IpNetwork,
    pub action: String,
    pub status: Option<String>,
    pub count: Option<i32>,
    pub creation: Option<NaiveDateTime>,
}
