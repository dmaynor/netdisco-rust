//! Node model - MAC addresses seen on switch ports.

use chrono::NaiveDateTime;
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// A network node (host) identified by MAC address on a switch port.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Node {
    /// MAC address of the node
    pub mac: String,
    /// Switch IP where this MAC was seen
    pub switch: IpNetwork,
    /// Port where this MAC was seen
    pub port: String,
    /// VLAN where this MAC was seen
    pub vlan: Option<String>,
    /// Whether this is the current/active entry
    pub active: Option<bool>,
    /// OUI prefix (first 3 bytes of MAC)
    pub oui: Option<String>,
    /// First time this MAC was seen on this port
    pub time_first: Option<NaiveDateTime>,
    /// Most recent time this MAC was seen
    pub time_recent: Option<NaiveDateTime>,
    /// Last time this entry was active
    pub time_last: Option<NaiveDateTime>,
}

impl Node {
    /// Extract OUI from MAC address.
    pub fn extract_oui(mac: &str) -> String {
        mac.replace([':', '-', '.'], "")
            .chars()
            .take(6)
            .collect::<String>()
            .to_uppercase()
    }
}
