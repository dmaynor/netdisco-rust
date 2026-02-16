//! NodeNbt model - NetBIOS information for nodes.

use chrono::NaiveDateTime;
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NodeNbt {
    pub mac: String,
    pub ip: Option<IpNetwork>,
    pub nbname: Option<String>,
    pub domain: Option<String>,
    pub server: Option<bool>,
    pub nbuser: Option<String>,
    pub active: Option<bool>,
    pub time_first: Option<NaiveDateTime>,
    pub time_last: Option<NaiveDateTime>,
}
