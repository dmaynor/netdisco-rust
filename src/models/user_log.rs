//! UserLog model - login/logout and change audit trail.

use chrono::NaiveDateTime;
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserLog {
    pub entry: Option<i32>,
    pub username: Option<String>,
    pub userip: Option<IpNetwork>,
    pub event: Option<String>,
    pub details: Option<String>,
    pub creation: Option<NaiveDateTime>,
}
