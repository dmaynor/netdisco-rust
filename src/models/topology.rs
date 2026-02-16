//! Topology model - manual device interconnections.

use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Topology {
    pub dev1: IpNetwork,
    pub port1: String,
    pub dev2: IpNetwork,
    pub port2: String,
}
