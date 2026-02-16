//! Unit tests for the configuration system.

use netdisco::config::settings::*;
use pretty_assertions::assert_eq;

// ==================== Default Config Tests ====================

#[test]
fn test_default_config_database() {
    let config = NetdiscoConfig::default();
    assert_eq!(config.database.name, "netdisco");
    assert_eq!(config.database.host, "localhost");
    assert_eq!(config.database.user, "netdisco");
    assert_eq!(config.database.pass, "");
    assert_eq!(config.database.port, 5432);
}

#[test]
fn test_default_config_web() {
    let config = NetdiscoConfig::default();
    assert_eq!(config.web_home, "/inventory");
    assert_eq!(config.path, "/");
    assert_eq!(config.branding_text, "Netdisco");
    assert!(config.navbar_autocomplete);
    assert_eq!(config.max_typeahead_rows, 50);
    assert_eq!(config.table_pagesize, 10);
}

#[test]
fn test_default_config_snmp() {
    let config = NetdiscoConfig::default();
    assert_eq!(config.snmpver, 3);
    assert_eq!(config.snmptimeout, 3_000_000);
    assert_eq!(config.snmpretries, 2);
    assert!(!config.bulkwalk_off);
    assert_eq!(config.bulkwalk_repeaters, 20);
}

#[test]
fn test_default_config_discovery() {
    let config = NetdiscoConfig::default();
    assert!(config.discover_neighbors);
    assert!(config.discover_routed_neighbors);
    assert!(config.discover_waps);
    assert!(!config.discover_phones);
    assert_eq!(config.discover_min_age, 0);
    assert!(config.devices_no.is_empty());
    assert!(config.devices_only.is_empty());
}

#[test]
fn test_default_config_macsuck() {
    let config = NetdiscoConfig::default();
    assert!(!config.macsuck_all_vlans);
    assert!(!config.macsuck_no_unnamed);
    assert!(!config.macsuck_bleed);
    assert_eq!(config.macsuck_min_age, 0);
}

#[test]
fn test_default_config_expiration() {
    let config = NetdiscoConfig::default();
    assert_eq!(config.expire_devices, 60);
    assert_eq!(config.expire_nodes, 90);
    assert_eq!(config.expire_nodes_archive, 60);
    assert_eq!(config.expire_jobs, 14);
    assert_eq!(config.expire_userlog, 365);
}

#[test]
fn test_default_config_workers() {
    let config = NetdiscoConfig::default();
    assert_eq!(config.workers.tasks, "AUTO * 2");
    assert_eq!(config.workers.timeout, 600);
    assert_eq!(config.workers.sleep_time, 1);
    assert_eq!(config.workers.max_deferrals, 10);
    assert_eq!(config.workers.retry_after, "7 days");
    assert_eq!(config.workers.queue, "PostgreSQL");
}

#[test]
fn test_default_config_auth() {
    let config = NetdiscoConfig::default();
    assert!(!config.no_auth);
    assert!(!config.suggest_guest);
    assert!(config.safe_password_store);
    assert!(!config.trust_remote_user);
    assert_eq!(config.api_token_lifetime, 3600);
}

#[test]
fn test_default_config_portcontrol() {
    let config = NetdiscoConfig::default();
    assert!(!config.portctl_nameonly);
    assert!(config.portctl_native_vlan);
    assert!(!config.portctl_nowaps);
    assert!(!config.portctl_nophones);
    assert!(!config.portctl_uplinks);
}

#[test]
fn test_default_config_nbtstat() {
    let config = NetdiscoConfig::default();
    assert_eq!(config.nbtstat_max_age, 7);
    assert!((config.nbtstat_interval - 0.02).abs() < f64::EPSILON);
    assert_eq!(config.nbtstat_response_timeout, 1);
}

#[test]
fn test_default_config_dns() {
    let config = NetdiscoConfig::default();
    assert_eq!(config.dns.max_outstanding, 50);
    assert_eq!(config.dns.hosts_file, "/etc/hosts");
    assert!(config.dns.no.is_empty());
}

#[test]
fn test_default_config_web_plugins() {
    let config = NetdiscoConfig::default();
    assert!(config.web_plugins.contains(&"Inventory".to_string()));
    assert!(config.web_plugins.contains(&"Search::Device".to_string()));
    assert!(config.web_plugins.contains(&"Search::Node".to_string()));
    assert!(config.web_plugins.contains(&"Device::Ports".to_string()));
    assert!(config.extra_web_plugins.is_empty());
}

#[test]
fn test_default_config_log_level() {
    let config = NetdiscoConfig::default();
    assert_eq!(config.log, "warning");
}

// ==================== Database Connection String ====================

#[test]
fn test_db_connection_string_default() {
    let db = DatabaseConfig::default();
    assert_eq!(db.connection_string(), "postgres://netdisco:@localhost:5432/netdisco");
}

#[test]
fn test_db_connection_string_custom() {
    let db = DatabaseConfig {
        name: "mydb".into(),
        host: "db.example.com".into(),
        user: "admin".into(),
        pass: "s3cret".into(),
        port: 5433,
    };
    assert_eq!(db.connection_string(), "postgres://admin:s3cret@db.example.com:5433/mydb");
}

#[test]
fn test_db_connection_string_with_special_chars() {
    let db = DatabaseConfig {
        name: "netdisco".into(),
        host: "10.0.0.1".into(),
        user: "netdisco".into(),
        pass: "pass@word".into(),
        port: 5432,
    };
    assert_eq!(db.connection_string(), "postgres://netdisco:pass@word@10.0.0.1:5432/netdisco");
}

// ==================== Config Override Tests ====================

#[test]
fn test_config_apply_overrides_database() {
    let mut config = NetdiscoConfig::default();
    let overrides: serde_yaml::Value = serde_yaml::from_str(r#"
        database:
            name: production_db
            host: db-prod.example.com
            user: prod_user
            pass: prod_pass
    "#).unwrap();

    config.apply_overrides(&overrides).unwrap();
    assert_eq!(config.database.name, "production_db");
    assert_eq!(config.database.host, "db-prod.example.com");
    assert_eq!(config.database.user, "prod_user");
    assert_eq!(config.database.pass, "prod_pass");
}

#[test]
fn test_config_apply_overrides_partial_database() {
    let mut config = NetdiscoConfig::default();
    let overrides: serde_yaml::Value = serde_yaml::from_str(r#"
        database:
            host: new-host.example.com
    "#).unwrap();

    config.apply_overrides(&overrides).unwrap();
    assert_eq!(config.database.host, "new-host.example.com");
    // Other fields should remain at defaults
    assert_eq!(config.database.name, "netdisco");
    assert_eq!(config.database.user, "netdisco");
    assert_eq!(config.database.port, 5432);
}

#[test]
fn test_config_apply_overrides_community() {
    let mut config = NetdiscoConfig::default();
    let overrides: serde_yaml::Value = serde_yaml::from_str(r#"
        community:
            - public
            - private
            - mycomm
    "#).unwrap();

    config.apply_overrides(&overrides).unwrap();
    assert_eq!(config.community, vec!["public", "private", "mycomm"]);
}

#[test]
fn test_config_apply_overrides_domain_suffix() {
    let mut config = NetdiscoConfig::default();
    let overrides: serde_yaml::Value = serde_yaml::from_str(r#"
        domain_suffix:
            - .example.com
            - .corp.local
    "#).unwrap();

    config.apply_overrides(&overrides).unwrap();
    assert_eq!(config.domain_suffix, vec![".example.com", ".corp.local"]);
}

#[test]
fn test_config_apply_overrides_no_auth() {
    let mut config = NetdiscoConfig::default();
    let overrides: serde_yaml::Value = serde_yaml::from_str(r#"
        no_auth: true
    "#).unwrap();

    config.apply_overrides(&overrides).unwrap();
    assert!(config.no_auth);
}

#[test]
fn test_config_apply_overrides_empty() {
    let mut config = NetdiscoConfig::default();
    let overrides: serde_yaml::Value = serde_yaml::from_str("---").unwrap();
    config.apply_overrides(&overrides).unwrap();
    // Should not change anything
    assert_eq!(config.database.name, "netdisco");
}

// ==================== Environment Variable Overrides ====================

#[test]
#[serial_test::serial]
fn test_config_env_override_db_name() {
    let mut config = NetdiscoConfig::default();
    std::env::set_var("NETDISCO_DB_NAME", "env_db");
    config.apply_env_overrides();
    assert_eq!(config.database.name, "env_db");
    std::env::remove_var("NETDISCO_DB_NAME");
}

#[test]
#[serial_test::serial]
fn test_config_env_override_db_host() {
    let mut config = NetdiscoConfig::default();
    std::env::set_var("NETDISCO_DB_HOST", "env-host.example.com");
    config.apply_env_overrides();
    assert_eq!(config.database.host, "env-host.example.com");
    std::env::remove_var("NETDISCO_DB_HOST");
}

#[test]
#[serial_test::serial]
fn test_config_env_override_db_user() {
    let mut config = NetdiscoConfig::default();
    std::env::set_var("NETDISCO_DB_USER", "env_user");
    config.apply_env_overrides();
    assert_eq!(config.database.user, "env_user");
    std::env::remove_var("NETDISCO_DB_USER");
}

#[test]
#[serial_test::serial]
fn test_config_env_override_db_pass() {
    let mut config = NetdiscoConfig::default();
    std::env::set_var("NETDISCO_DB_PASS", "env_pass");
    config.apply_env_overrides();
    assert_eq!(config.database.pass, "env_pass");
    std::env::remove_var("NETDISCO_DB_PASS");
}

#[test]
#[serial_test::serial]
fn test_config_env_override_community() {
    let mut config = NetdiscoConfig::default();
    std::env::set_var("NETDISCO_RO_COMMUNITY", "comm1,comm2,comm3");
    config.apply_env_overrides();
    assert_eq!(config.community, vec!["comm1", "comm2", "comm3"]);
    std::env::remove_var("NETDISCO_RO_COMMUNITY");
}

#[test]
#[serial_test::serial]
fn test_config_env_override_domain() {
    let mut config = NetdiscoConfig::default();
    std::env::set_var("NETDISCO_DOMAIN", ".test.local");
    config.apply_env_overrides();
    assert_eq!(config.domain_suffix, vec![".test.local"]);
    std::env::remove_var("NETDISCO_DOMAIN");
}

#[test]
#[serial_test::serial]
fn test_config_env_override_no_vars_set() {
    // Clean env of any NETDISCO_ vars that could interfere
    for key in ["NETDISCO_DB_NAME", "NETDISCO_DB_HOST", "NETDISCO_DB_USER",
                "NETDISCO_DB_PASS", "NETDISCO_RO_COMMUNITY", "NETDISCO_DOMAIN"] {
        std::env::remove_var(key);
    }
    let mut config = NetdiscoConfig::default();
    config.apply_env_overrides();
    assert_eq!(config.database.name, "netdisco");
    assert_eq!(config.database.host, "localhost");
}

// ==================== YAML Deserialization Tests ====================

#[test]
fn test_config_from_yaml_minimal() {
    let yaml = r#"
        database:
            name: testdb
    "#;
    let config: NetdiscoConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.database.name, "testdb");
    // Defaults should still apply
    assert_eq!(config.log, "warning");
}

#[test]
fn test_config_from_yaml_full() {
    let yaml = r#"
        log: debug
        no_auth: true
        snmpver: 2
        snmpretries: 5
        expire_devices: 30
        database:
            name: fulltest
            host: myhost
            port: 5433
    "#;
    let config: NetdiscoConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.log, "debug");
    assert!(config.no_auth);
    assert_eq!(config.snmpver, 2);
    assert_eq!(config.snmpretries, 5);
    assert_eq!(config.expire_devices, 30);
    assert_eq!(config.database.name, "fulltest");
    assert_eq!(config.database.host, "myhost");
    assert_eq!(config.database.port, 5433);
}

#[test]
fn test_workers_config_from_yaml() {
    let yaml = r#"
        tasks: "8"
        timeout: 300
        sleep_time: 2
        max_deferrals: 5
    "#;
    let config: WorkersConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.tasks, "8");
    assert_eq!(config.timeout, 300);
    assert_eq!(config.sleep_time, 2);
    assert_eq!(config.max_deferrals, 5);
}

#[test]
fn test_schedule_config_default() {
    let config = ScheduleConfig::default();
    assert!(config.discoverall.is_none());
    assert!(config.macwalk.is_none());
    assert!(config.arpwalk.is_none());
    assert!(config.nbtwalk.is_none());
    assert!(config.expire.is_none());
}

// ==================== Config File Loading Tests ====================

#[test]
fn test_load_config_nonexistent_dir() {
    let result = netdisco::config::load_config(Some(std::path::Path::new("/nonexistent/path/that/doesnt/exist")));
    // Should still succeed with defaults since config.yml is optional
    let config = result.unwrap();
    assert_eq!(config.database.name, "netdisco");
}

#[test]
fn test_load_config_with_tempdir() {
    let tmpdir = tempfile::tempdir().unwrap();

    // Create a config.yml in the temp dir
    let config_path = tmpdir.path().join("config.yml");
    std::fs::write(&config_path, r#"
        database:
            name: tmptest
            host: tmphost
        snmpver: 2
    "#).unwrap();

    let result = netdisco::config::load_config(Some(tmpdir.path()));
    let config = result.unwrap();
    assert_eq!(config.database.name, "tmptest");
    assert_eq!(config.database.host, "tmphost");
    assert_eq!(config.snmpver, 2);
}

#[test]
fn test_load_config_with_deployment_override() {
    let tmpdir = tempfile::tempdir().unwrap();

    // Create config.yml
    std::fs::write(tmpdir.path().join("config.yml"), r#"
        database:
            name: base_db
            host: base_host
    "#).unwrap();

    // Create environments/deployment.yml
    let env_dir = tmpdir.path().join("environments");
    std::fs::create_dir_all(&env_dir).unwrap();
    std::fs::write(env_dir.join("deployment.yml"), r#"
        database:
            host: override_host
            pass: secret123
    "#).unwrap();

    let result = netdisco::config::load_config(Some(tmpdir.path()));
    let config = result.unwrap();
    assert_eq!(config.database.name, "base_db"); // Not overridden
    assert_eq!(config.database.host, "override_host"); // Overridden
    assert_eq!(config.database.pass, "secret123"); // New from override
}
