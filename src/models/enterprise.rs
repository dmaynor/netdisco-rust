//! Enterprise (SNMP enterprise OID) model.
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Enterprise {
    pub id: i32,
    pub name: Option<String>,
}
