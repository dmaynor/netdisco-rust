//! NodeWireless model - wireless client association information.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NodeWireless {
    pub mac: String,
    pub ssid: Option<String>,
    pub uptime: Option<i32>,
    pub maxrate: Option<i32>,
    pub txrate: Option<i32>,
    pub sigstrength: Option<i32>,
    pub sigqual: Option<i32>,
    pub rxpkt: Option<i32>,
    pub txpkt: Option<i32>,
    pub rxbyte: Option<i64>,
    pub txbyte: Option<i64>,
    pub time_last: Option<NaiveDateTime>,
}
