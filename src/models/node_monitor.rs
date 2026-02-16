//! NodeMonitor model - monitored MAC addresses.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NodeMonitor {
    pub mac: String,
    pub active: Option<bool>,
    pub why: Option<String>,
    pub cc: Option<String>,
    pub date: Option<NaiveDateTime>,
}
