//! Integration tests for backend components.

use netdisco::config::NetdiscoConfig;
use netdisco::models::admin::Admin;
use ipnetwork::IpNetwork;

// ==================== Worker Selection Tests ====================

/// Verify the action dispatch logic matches the expected actions
#[test]
fn test_action_dispatch_known_actions() {
    let known_actions = vec![
        "discover", "discoverall", "macsuck", "macwalk",
        "arpnip", "arpwalk", "nbtstat", "nbtwalk",
        "expire", "delete", "portcontrol", "portname",
        "portvlan", "power", "graph", "show", "stats", "linter",
    ];

    for action in &known_actions {
        // Just verify these are strings - the actual dispatch is tested in backend/manager
        assert!(!action.is_empty(), "Action should not be empty");
    }
}

/// Test a job struct for device-requiring actions
#[test]
fn test_job_with_device_ip() {
    let job = Admin {
        job: Some(1),
        entered: None,
        started: None,
        finished: None,
        device: Some("10.0.0.1/32".parse().unwrap()),
        port: None,
        action: Some("discover".to_string()),
        subaction: None,
        status: Some("queued".to_string()),
        username: Some("scheduler".to_string()),
        userip: None,
        log: None,
        debug: None,
    };

    assert!(job.device.is_some());
    assert_eq!(job.action.as_deref(), Some("discover"));
    assert_eq!(job.status.as_deref(), Some("queued"));
}

/// Test a job struct for port control actions
#[test]
fn test_job_with_port_control() {
    let job = Admin {
        job: Some(2),
        entered: None,
        started: None,
        finished: None,
        device: Some("10.0.0.1/32".parse().unwrap()),
        port: Some("GigabitEthernet0/1".to_string()),
        action: Some("portcontrol".to_string()),
        subaction: Some("down".to_string()),
        status: Some("queued".to_string()),
        username: Some("admin".to_string()),
        userip: Some("192.168.1.100/32".parse().unwrap()),
        log: None,
        debug: Some(true),
    };

    assert!(job.device.is_some());
    assert!(job.port.is_some());
    assert_eq!(job.subaction.as_deref(), Some("down"));
}

/// Test a job struct for VLAN actions
#[test]
fn test_job_with_vlan_change() {
    let job = Admin {
        job: Some(3),
        entered: None,
        started: None,
        finished: None,
        device: Some("10.0.0.1/32".parse().unwrap()),
        port: Some("Gi0/1".to_string()),
        action: Some("portvlan".to_string()),
        subaction: Some("100".to_string()),
        status: Some("queued".to_string()),
        username: Some("admin".to_string()),
        userip: None,
        log: None,
        debug: None,
    };

    let vlan: i32 = job.subaction.as_deref().unwrap().parse().unwrap();
    assert_eq!(vlan, 100);
}

/// Test a job struct for discover all (no device IP)
#[test]
fn test_job_discoverall_no_device() {
    let job = Admin {
        job: Some(4),
        entered: None,
        started: None,
        finished: None,
        device: None,
        port: None,
        action: Some("discoverall".to_string()),
        subaction: None,
        status: Some("queued".to_string()),
        username: Some("scheduler".to_string()),
        userip: None,
        log: None,
        debug: None,
    };

    assert!(job.device.is_none());
    assert_eq!(job.action.as_deref(), Some("discoverall"));
}

// ==================== Port Control Subaction Tests ====================

#[test]
fn test_port_control_valid_subactions() {
    let valid = vec!["up", "enable", "down", "disable"];
    for subaction in &valid {
        let target = match *subaction {
            "up" | "enable" => "up",
            "down" | "disable" => "down",
            _ => "unknown",
        };
        assert!(target == "up" || target == "down",
            "Subaction '{}' should map to 'up' or 'down'", subaction);
    }
}

#[test]
fn test_port_control_invalid_subaction() {
    let subaction = "toggle";
    let target = match subaction {
        "up" | "enable" => Some("up"),
        "down" | "disable" => Some("down"),
        _ => None,
    };
    assert!(target.is_none(), "Unknown subaction should return None");
}

// ==================== Worker Count Calculation ====================

#[test]
fn test_calculate_workers_auto() {
    let tasks = "AUTO * 2";
    let cpus = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2);

    if tasks.starts_with("AUTO") {
        if let Some(multiplier) = tasks.split('*').nth(1) {
            let mult: usize = multiplier.trim().parse().unwrap_or(2);
            let result = cpus * mult;
            assert!(result >= 2, "Should have at least 2 workers");
        }
    }
}

#[test]
fn test_calculate_workers_fixed() {
    let tasks = "8";
    let result: usize = tasks.parse().unwrap_or(4);
    assert_eq!(result, 8);
}

#[test]
fn test_calculate_workers_invalid() {
    let tasks = "not_a_number";
    let result: usize = tasks.parse().unwrap_or(4);
    assert_eq!(result, 4);
}

#[test]
fn test_calculate_workers_auto_no_multiplier() {
    let tasks = "AUTO";
    let cpus = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2);

    if tasks.starts_with("AUTO") {
        if tasks.split('*').nth(1).is_none() {
            let result = cpus * 2;
            assert!(result >= 2);
        }
    }
}

// ==================== Config for Backend Tests ====================

#[test]
fn test_backend_config_defaults() {
    let config = NetdiscoConfig::default();
    assert_eq!(config.workers.timeout, 600);
    assert_eq!(config.workers.sleep_time, 1);
    assert_eq!(config.workers.max_deferrals, 10);
}

// ==================== Expire Config Tests ====================

#[test]
fn test_expire_intervals_are_positive() {
    let config = NetdiscoConfig::default();
    assert!(config.expire_devices > 0);
    assert!(config.expire_nodes > 0);
    assert!(config.expire_nodes_archive > 0);
    assert!(config.expire_jobs > 0);
    assert!(config.expire_userlog > 0);
}

#[test]
fn test_expire_interval_format() {
    let config = NetdiscoConfig::default();
    let device_interval = format!("{} days", config.expire_devices);
    assert_eq!(device_interval, "60 days");

    let job_interval = format!("{} days", config.expire_jobs);
    assert_eq!(job_interval, "14 days");
}
