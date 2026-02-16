//! Remaining smaller models.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Log {
    pub id: Option<i32>,
    pub creation: Option<NaiveDateTime>,
    pub class: Option<String>,
    pub entry: Option<String>,
    pub logfile: Option<String>,
}
