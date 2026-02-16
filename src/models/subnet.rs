//! Subnet model.

use chrono::NaiveDateTime;
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Subnet {
    pub net: IpNetwork,
    pub creation: Option<NaiveDateTime>,
    pub last_discover: Option<NaiveDateTime>,
}
