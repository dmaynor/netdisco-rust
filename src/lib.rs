//! Netdisco - Network Management Tool
//!
//! A Rust port of the App::Netdisco Perl application for network device
//! discovery, monitoring, and management via SNMP, ARP/NDP, and LLDP/CDP.

pub mod config;
pub mod db;
pub mod models;
pub mod snmp;
pub mod web;
pub mod backend;
pub mod worker;
pub mod util;

/// Application version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default web server port.
pub const DEFAULT_WEB_PORT: u16 = 5000;

/// Default backend port.
pub const DEFAULT_BACKEND_PORT: u16 = 5001;

/// Default configuration file name.
pub const DEFAULT_CONFIG_FILE: &str = "config.yml";
