//! Configuration settings structures.
//!
//! These structs represent the full Netdisco configuration tree,
//! matching the keys in config.yml / deployment.yml.

use serde::{Deserialize, Serialize};
use anyhow::Result;

/// Top-level Netdisco configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NetdiscoConfig {
    // General
    pub log: String,
    pub domain_suffix: Vec<String>,

    // Database
    pub database: DatabaseConfig,

    // Web frontend
    pub no_auth: bool,
    pub suggest_guest: bool,
    pub branding_text: String,
    pub navbar_autocomplete: bool,
    pub max_typeahead_rows: usize,
    pub web_home: String,
    pub path: String,
    pub safe_password_store: bool,
    pub table_pagesize: usize,
    pub web_plugins: Vec<String>,
    pub extra_web_plugins: Vec<String>,

    // SNMP
    pub community: Vec<String>,
    pub community_rw: Vec<String>,
    pub snmpver: u8,
    pub snmptimeout: u64,
    pub snmpretries: u32,
    pub bulkwalk_off: bool,
    pub bulkwalk_repeaters: u32,

    // Discovery control
    pub devices_no: Vec<String>,
    pub devices_only: Vec<String>,
    pub discover_no: Vec<String>,
    pub discover_only: Vec<String>,
    pub discover_neighbors: bool,
    pub discover_routed_neighbors: bool,
    pub discover_waps: bool,
    pub discover_phones: bool,
    pub discover_min_age: u64,

    // MAC/ARP collection
    pub macsuck_no: Vec<String>,
    pub macsuck_only: Vec<String>,
    pub macsuck_all_vlans: bool,
    pub macsuck_no_unnamed: bool,
    pub macsuck_bleed: bool,
    pub macsuck_min_age: u64,
    pub arpnip_no: Vec<String>,
    pub arpnip_only: Vec<String>,
    pub arpnip_min_age: u64,

    // NetBIOS
    pub nbtstat_no: Vec<String>,
    pub nbtstat_only: Vec<String>,
    pub nbtstat_max_age: u32,
    pub nbtstat_interval: f64,
    pub nbtstat_response_timeout: u32,

    // Expiration
    pub expire_devices: u32,
    pub expire_nodes: u32,
    pub expire_nodes_archive: u32,
    pub expire_jobs: u32,
    pub expire_userlog: u32,
    pub node_freshness: u32,

    // Workers / Backend
    pub workers: WorkersConfig,
    pub schedule: ScheduleConfig,

    // DNS
    pub dns: DnsConfig,

    // Port control
    pub portctl_nameonly: bool,
    pub portctl_native_vlan: bool,
    pub portctl_nowaps: bool,
    pub portctl_nophones: bool,
    pub portctl_uplinks: bool,

    // Authentication
    pub ldap: LdapConfig,
    pub radius: RadiusConfig,
    pub tacacs: TacacsConfig,
    pub trust_remote_user: bool,
    pub api_token_lifetime: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DatabaseConfig {
    pub name: String,
    pub host: String,
    pub user: String,
    pub pass: String,
    pub port: u16,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            name: "netdisco".into(),
            host: "localhost".into(),
            user: "netdisco".into(),
            pass: "".into(),
            port: 5432,
        }
    }
}

impl DatabaseConfig {
    /// Build a PostgreSQL connection string.
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user, self.pass, self.host, self.port, self.name
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkersConfig {
    pub tasks: String,
    pub timeout: u64,
    pub sleep_time: u64,
    pub min_runtime: u64,
    pub max_deferrals: u32,
    pub retry_after: String,
    pub queue: String,
}

impl Default for WorkersConfig {
    fn default() -> Self {
        Self {
            tasks: "AUTO * 2".into(),
            timeout: 600,
            sleep_time: 1,
            min_runtime: 0,
            max_deferrals: 10,
            retry_after: "7 days".into(),
            queue: "PostgreSQL".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ScheduleConfig {
    pub discoverall: Option<ScheduleEntry>,
    pub macwalk: Option<ScheduleEntry>,
    pub arpwalk: Option<ScheduleEntry>,
    pub nbtwalk: Option<ScheduleEntry>,
    pub expire: Option<ScheduleEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleEntry {
    pub when: serde_yaml::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DnsConfig {
    pub max_outstanding: u32,
    pub hosts_file: String,
    pub no: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct LdapConfig {
    pub servers: Vec<String>,
    pub user_string: String,
    pub base: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RadiusConfig {
    pub servers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TacacsConfig {
    pub servers: Vec<String>,
}

impl Default for NetdiscoConfig {
    fn default() -> Self {
        Self {
            log: "warning".into(),
            domain_suffix: vec![],
            database: DatabaseConfig::default(),
            no_auth: false,
            suggest_guest: false,
            branding_text: "Netdisco".into(),
            navbar_autocomplete: true,
            max_typeahead_rows: 50,
            web_home: "/inventory".into(),
            path: "/".into(),
            safe_password_store: true,
            table_pagesize: 10,
            web_plugins: vec![
                "Inventory".into(),
                "Search::Device".into(),
                "Search::Node".into(),
                "Search::VLAN".into(),
                "Search::Port".into(),
                "Device::Details".into(),
                "Device::Ports".into(),
            ],
            extra_web_plugins: vec![],
            community: vec![],
            community_rw: vec![],
            snmpver: 3,
            snmptimeout: 3_000_000,
            snmpretries: 2,
            bulkwalk_off: false,
            bulkwalk_repeaters: 20,
            devices_no: vec![],
            devices_only: vec![],
            discover_no: vec![],
            discover_only: vec![],
            discover_neighbors: true,
            discover_routed_neighbors: true,
            discover_waps: true,
            discover_phones: false,
            discover_min_age: 0,
            macsuck_no: vec![],
            macsuck_only: vec![],
            macsuck_all_vlans: false,
            macsuck_no_unnamed: false,
            macsuck_bleed: false,
            macsuck_min_age: 0,
            arpnip_no: vec![],
            arpnip_only: vec![],
            arpnip_min_age: 0,
            nbtstat_no: vec![],
            nbtstat_only: vec![],
            nbtstat_max_age: 7,
            nbtstat_interval: 0.02,
            nbtstat_response_timeout: 1,
            expire_devices: 60,
            expire_nodes: 90,
            expire_nodes_archive: 60,
            expire_jobs: 14,
            expire_userlog: 365,
            node_freshness: 0,
            workers: WorkersConfig::default(),
            schedule: ScheduleConfig::default(),
            dns: DnsConfig {
                max_outstanding: 50,
                hosts_file: "/etc/hosts".into(),
                no: vec![],
            },
            portctl_nameonly: false,
            portctl_native_vlan: true,
            portctl_nowaps: false,
            portctl_nophones: false,
            portctl_uplinks: false,
            ldap: LdapConfig::default(),
            radius: RadiusConfig::default(),
            tacacs: TacacsConfig::default(),
            trust_remote_user: false,
            api_token_lifetime: 3600,
        }
    }
}

impl NetdiscoConfig {
    /// Apply overrides from a YAML value (deployment.yml).
    pub fn apply_overrides(&mut self, overrides: &serde_yaml::Value) -> Result<()> {
        // Merge database settings
        if let Some(db) = overrides.get("database") {
            if let Some(name) = db.get("name").and_then(|v| v.as_str()) {
                self.database.name = name.to_string();
            }
            if let Some(host) = db.get("host").and_then(|v| v.as_str()) {
                self.database.host = host.to_string();
            }
            if let Some(user) = db.get("user").and_then(|v| v.as_str()) {
                self.database.user = user.to_string();
            }
            if let Some(pass) = db.get("pass").and_then(|v| v.as_str()) {
                self.database.pass = pass.to_string();
            }
        }

        // Merge scalar overrides
        if let Some(v) = overrides.get("community") {
            if let Ok(communities) = serde_yaml::from_value::<Vec<String>>(v.clone()) {
                self.community = communities;
            }
        }
        if let Some(v) = overrides.get("domain_suffix") {
            if let Ok(suffixes) = serde_yaml::from_value::<Vec<String>>(v.clone()) {
                self.domain_suffix = suffixes;
            }
        }
        if let Some(v) = overrides.get("no_auth").and_then(|v| v.as_bool()) {
            self.no_auth = v;
        }

        Ok(())
    }

    /// Apply environment variable overrides (for Docker compatibility).
    pub fn apply_env_overrides(&mut self) {
        if let Ok(v) = std::env::var("NETDISCO_DB_NAME") {
            self.database.name = v;
        }
        if let Ok(v) = std::env::var("NETDISCO_DB_HOST") {
            self.database.host = v;
        }
        if let Ok(v) = std::env::var("NETDISCO_DB_USER") {
            self.database.user = v;
        }
        if let Ok(v) = std::env::var("NETDISCO_DB_PASS") {
            self.database.pass = v;
        }
        if let Ok(v) = std::env::var("NETDISCO_RO_COMMUNITY") {
            self.community = v.split(',').map(String::from).collect();
        }
        if let Ok(v) = std::env::var("NETDISCO_DOMAIN") {
            self.domain_suffix = vec![v];
        }
    }
}
