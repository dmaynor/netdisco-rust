//! Statistics model.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Statistics {
    pub key: String,
    pub value: Option<String>,
    pub creation: Option<NaiveDateTime>,
}
