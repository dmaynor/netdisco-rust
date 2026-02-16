//! Session model for web sessions.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: String,
    pub creation: Option<NaiveDateTime>,
    pub a_session: Option<String>,
}
