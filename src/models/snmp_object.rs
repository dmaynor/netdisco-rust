//! SNMP Object model - custom SNMP OID definitions.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SnmpObject {
    pub oid: String,
    pub oid_name: Option<String>,
    pub mib: Option<String>,
    pub leaf: Option<bool>,
}
