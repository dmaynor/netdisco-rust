//! NodeIp model - IP to MAC address mappings from ARP/NDP tables.

use chrono::NaiveDateTime;
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NodeIp {
    /// MAC address
    pub mac: String,
    /// IP address
    pub ip: IpNetwork,
    /// Whether this is the current/active entry
    pub active: Option<bool>,
    /// First time seen
    pub time_first: Option<NaiveDateTime>,
    /// Last time seen
    pub time_last: Option<NaiveDateTime>,
    /// DNS name
    pub dns: Option<String>,
}
