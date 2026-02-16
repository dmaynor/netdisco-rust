//! Manufacturer model.
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Manufacturer {
    pub id: Option<i32>,
    pub name: Option<String>,
}
