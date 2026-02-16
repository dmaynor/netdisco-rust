//! User model - authentication and authorization.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub username: String,
    pub password: Option<String>,
    pub creation: Option<NaiveDateTime>,
    pub last_on: Option<NaiveDateTime>,
    pub port_control: Option<bool>,
    pub ldap: Option<bool>,
    pub admin: Option<bool>,
    pub fullname: Option<String>,
    pub note: Option<String>,
}

impl User {
    /// Check if this user has admin privileges.
    pub fn is_admin(&self) -> bool {
        self.admin.unwrap_or(false)
    }

    /// Check if user has port control privileges.
    pub fn has_port_control(&self) -> bool {
        self.port_control.unwrap_or(false)
    }
}
