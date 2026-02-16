//! Admin model - job queue for backend tasks.

use chrono::NaiveDateTime;
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// A job in the admin queue (backend task).
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Admin {
    /// Auto-increment job ID
    pub job: Option<i32>,
    /// When the job was queued
    pub entered: Option<NaiveDateTime>,
    /// When the job started executing
    pub started: Option<NaiveDateTime>,
    /// When the job finished
    pub finished: Option<NaiveDateTime>,
    /// Target device IP
    pub device: Option<IpNetwork>,
    /// Target port (for port-level actions)
    pub port: Option<String>,
    /// Action to perform (e.g., "discover", "macsuck", "arpnip")
    pub action: Option<String>,
    /// Sub-action detail
    pub subaction: Option<String>,
    /// Job status (queued, running, done, error)
    pub status: Option<String>,
    /// Username who requested the job
    pub username: Option<String>,
    /// IP of user who requested the job
    pub userip: Option<IpNetwork>,
    /// Result/error log
    pub log: Option<String>,
    /// Enable debug logging
    pub debug: Option<bool>,
}

/// Status values for admin jobs.
pub mod status {
    pub const QUEUED: &str = "queued";
    pub const RUNNING: &str = "running";
    pub const DONE: &str = "done";
    pub const ERROR: &str = "error";
    pub const DEFERRED: &str = "deferred";
}
