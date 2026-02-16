//! OUI (vendor MAC prefix) model.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Oui {
    pub oui: String,
    pub company: Option<String>,
}
