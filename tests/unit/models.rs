//! Unit tests for all model types.

use netdisco::models::device::Device;
use netdisco::models::device_port::{DevicePort, NewDevicePort};
use netdisco::models::node::Node;
use netdisco::models::user::User;
use netdisco::models::admin::{self, Admin};
use ipnetwork::IpNetwork;
use chrono::NaiveDateTime;
use pretty_assertions::assert_eq;

// ==================== Device Model Tests ====================

fn make_test_device(layers: Option<&str>) -> Device {
    Device {
        ip: "192.168.1.1/32".parse().unwrap(),
        creation: None,
        dns: None,
        description: None,
        uptime: None,
        contact: None,
        name: None,
        location: None,
        layers: layers.map(String::from),
        ports: None,
        mac: None,
        serial: None,
        model: None,
        ps1_type: None,
        ps2_type: None,
        ps1_status: None,
        ps2_status: None,
        fan: None,
        slots: None,
        vendor: None,
        os: None,
        os_ver: None,
        log: None,
        snmp_ver: None,
        snmp_comm: None,
        snmp_class: None,
        vtp_domain: None,
        last_discover: None,
        last_macsuck: None,
        last_arpnip: None,
        pae_is_enabled: None,
        custom_fields: None,
        tags: None,
    }
}

#[test]
fn test_device_has_layer_2() {
    let device = make_test_device(Some("0000010"));
    assert!(device.has_layer(2));
    assert!(!device.has_layer(3));
    assert!(!device.has_layer(1));
}

#[test]
fn test_device_has_layer_3() {
    let device = make_test_device(Some("0000100"));
    assert!(device.has_layer(3));
    assert!(!device.has_layer(2));
}

#[test]
fn test_device_has_layer_7() {
    let device = make_test_device(Some("1000000"));
    assert!(device.has_layer(7));
    assert!(!device.has_layer(1));
}

#[test]
fn test_device_has_layer_multibit() {
    // Layer 2 + Layer 3
    let device = make_test_device(Some("0000110"));
    assert!(device.has_layer(2));
    assert!(device.has_layer(3));
    assert!(!device.has_layer(4));
}

#[test]
fn test_device_has_layer_all() {
    let device = make_test_device(Some("1111111"));
    for layer in 1..=7 {
        assert!(device.has_layer(layer), "Layer {} should be set", layer);
    }
}

#[test]
fn test_device_has_layer_none_set() {
    let device = make_test_device(Some("0000000"));
    for layer in 1..=7 {
        assert!(!device.has_layer(layer), "Layer {} should NOT be set", layer);
    }
}

#[test]
fn test_device_has_layer_boundary_zero() {
    let device = make_test_device(Some("1111111"));
    assert!(!device.has_layer(0), "Layer 0 is not valid");
}

#[test]
fn test_device_has_layer_boundary_eight() {
    let device = make_test_device(Some("1111111"));
    assert!(!device.has_layer(8), "Layer 8 is not valid");
}

#[test]
fn test_device_has_layer_none_layers() {
    let device = make_test_device(None);
    assert!(!device.has_layer(2));
    assert!(!device.has_layer(3));
}

#[test]
fn test_device_is_switch() {
    let device = make_test_device(Some("0000010"));
    assert!(device.is_switch());
    assert!(!device.is_router());
}

#[test]
fn test_device_is_router() {
    let device = make_test_device(Some("0000100"));
    assert!(device.is_router());
    assert!(!device.is_switch());
}

#[test]
fn test_device_is_switch_and_router() {
    let device = make_test_device(Some("0000110"));
    assert!(device.is_switch());
    assert!(device.is_router());
}

#[test]
fn test_device_display_name_prefers_dns() {
    let mut device = make_test_device(None);
    device.dns = Some("switch1.example.com".to_string());
    device.name = Some("SWITCH1".to_string());
    assert_eq!(device.display_name(), "switch1.example.com");
}

#[test]
fn test_device_display_name_falls_back_to_name() {
    let mut device = make_test_device(None);
    device.dns = None;
    device.name = Some("SWITCH1".to_string());
    assert_eq!(device.display_name(), "SWITCH1");
}

#[test]
fn test_device_display_name_falls_back_to_ip() {
    let device = make_test_device(None);
    assert_eq!(device.display_name(), "192.168.1.1/32");
}

#[test]
fn test_device_display_name_empty_dns() {
    let mut device = make_test_device(None);
    device.dns = Some("".to_string());
    device.name = Some("SWITCH1".to_string());
    assert_eq!(device.display_name(), "SWITCH1");
}

#[test]
fn test_device_display_name_empty_name() {
    let mut device = make_test_device(None);
    device.dns = Some("".to_string());
    device.name = Some("".to_string());
    assert_eq!(device.display_name(), "192.168.1.1/32");
}

// ==================== Node Model Tests ====================

#[test]
fn test_extract_oui_standard_format() {
    assert_eq!(Node::extract_oui("00:11:22:33:44:55"), "001122");
}

#[test]
fn test_extract_oui_dash_format() {
    assert_eq!(Node::extract_oui("00-11-22-33-44-55"), "001122");
}

#[test]
fn test_extract_oui_dot_format() {
    assert_eq!(Node::extract_oui("0011.2233.4455"), "001122");
}

#[test]
fn test_extract_oui_compact_format() {
    assert_eq!(Node::extract_oui("001122334455"), "001122");
}

#[test]
fn test_extract_oui_lowercase() {
    assert_eq!(Node::extract_oui("aa:bb:cc:dd:ee:ff"), "AABBCC");
}

#[test]
fn test_extract_oui_mixed_case() {
    assert_eq!(Node::extract_oui("Aa:Bb:Cc:Dd:Ee:Ff"), "AABBCC");
}

// ==================== User Model Tests ====================

fn make_test_user(is_admin: Option<bool>, port_control: Option<bool>) -> User {
    User {
        username: "testuser".to_string(),
        password: None,
        creation: None,
        last_on: None,
        port_control,
        ldap: None,
        admin: is_admin,
        fullname: None,
        note: None,
    }
}

#[test]
fn test_user_is_admin_true() {
    let user = make_test_user(Some(true), None);
    assert!(user.is_admin());
}

#[test]
fn test_user_is_admin_false() {
    let user = make_test_user(Some(false), None);
    assert!(!user.is_admin());
}

#[test]
fn test_user_is_admin_none() {
    let user = make_test_user(None, None);
    assert!(!user.is_admin());
}

#[test]
fn test_user_has_port_control_true() {
    let user = make_test_user(None, Some(true));
    assert!(user.has_port_control());
}

#[test]
fn test_user_has_port_control_false() {
    let user = make_test_user(None, Some(false));
    assert!(!user.has_port_control());
}

#[test]
fn test_user_has_port_control_none() {
    let user = make_test_user(None, None);
    assert!(!user.has_port_control());
}

// ==================== Admin Status Constants ====================

#[test]
fn test_admin_status_constants() {
    assert_eq!(admin::status::QUEUED, "queued");
    assert_eq!(admin::status::RUNNING, "running");
    assert_eq!(admin::status::DONE, "done");
    assert_eq!(admin::status::ERROR, "error");
    assert_eq!(admin::status::DEFERRED, "deferred");
}

// ==================== Model Serialization Tests ====================

#[test]
fn test_device_serialization_roundtrip() {
    let device = make_test_device(Some("0000110"));
    let json = serde_json::to_string(&device).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["layers"], "0000110");
}

#[test]
fn test_user_serialization_roundtrip() {
    let user = make_test_user(Some(true), Some(false));
    let json = serde_json::to_string(&user).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["username"], "testuser");
    assert_eq!(parsed["admin"], true);
    assert_eq!(parsed["port_control"], false);
}

#[test]
fn test_admin_serialization() {
    let admin = Admin {
        job: Some(42),
        entered: None,
        started: None,
        finished: None,
        device: Some("10.0.0.1/32".parse().unwrap()),
        port: Some("Gi0/1".to_string()),
        action: Some("discover".to_string()),
        subaction: None,
        status: Some("queued".to_string()),
        username: Some("admin".to_string()),
        userip: None,
        log: None,
        debug: Some(false),
    };
    let json = serde_json::to_string(&admin).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["job"], 42);
    assert_eq!(parsed["action"], "discover");
    assert_eq!(parsed["status"], "queued");
}

#[test]
fn test_node_serialization() {
    let node = Node {
        mac: "00:11:22:33:44:55".into(),
        switch: "192.168.1.1/32".parse().unwrap(),
        port: "Gi0/1".into(),
        vlan: Some("100".into()),
        active: Some(true),
        oui: Some("001122".into()),
        time_first: None,
        time_recent: None,
        time_last: None,
    };
    let json = serde_json::to_string(&node).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["mac"], "00:11:22:33:44:55");
    assert_eq!(parsed["port"], "Gi0/1");
    assert_eq!(parsed["active"], true);
}

// ==================== NewDevice / NewDevicePort Tests ====================

#[test]
fn test_new_device_serialization() {
    let new_device = netdisco::models::device::NewDevice {
        ip: "10.0.0.1/32".parse().unwrap(),
        dns: Some("switch.example.com".into()),
        description: Some("Cisco IOS".into()),
        uptime: Some(1234567),
        contact: Some("admin@example.com".into()),
        name: Some("CORE-SW".into()),
        location: Some("DC1 Rack 5".into()),
        layers: Some("0000110".into()),
        ports: Some(48),
        mac: Some("00:aa:bb:cc:dd:ee".into()),
        serial: Some("FHK12345678".into()),
        model: Some("WS-C3850-48T".into()),
        vendor: Some("cisco".into()),
        os: Some("ios".into()),
        os_ver: Some("16.9.4".into()),
        snmp_ver: Some(2),
        snmp_comm: Some("public".into()),
        snmp_class: Some("SNMP::Info::Layer3::C3550".into()),
    };
    let json = serde_json::to_string(&new_device).unwrap();
    assert!(json.contains("WS-C3850-48T"));
    assert!(json.contains("CORE-SW"));
}

#[test]
fn test_new_device_port_serialization() {
    let port = NewDevicePort {
        ip: "10.0.0.1/32".parse().unwrap(),
        port: "GigabitEthernet0/1".into(),
        descr: Some("Server Port".into()),
        up: Some("up".into()),
        up_admin: Some("up".into()),
        port_type: Some("ethernetCsmacd".into()),
        duplex: Some("full".into()),
        speed: Some("1000000000".into()),
        name: Some("ToServer1".into()),
        mac: Some("00:11:22:33:44:55".into()),
        mtu: Some(1500),
        pvid: Some(100),
    };
    let json = serde_json::to_string(&port).unwrap();
    assert!(json.contains("GigabitEthernet0/1"));
    assert!(json.contains("1000000000"));
}

// ==================== IP Parsing Tests ====================

#[test]
fn test_ip_network_parsing_v4() {
    let ip: IpNetwork = "192.168.1.1/32".parse().unwrap();
    assert_eq!(ip.ip().to_string(), "192.168.1.1");
}

#[test]
fn test_ip_network_parsing_v4_subnet() {
    let ip: IpNetwork = "10.0.0.0/24".parse().unwrap();
    assert_eq!(ip.prefix(), 24);
}

#[test]
fn test_ip_network_parsing_v6() {
    let ip: IpNetwork = "::1/128".parse().unwrap();
    assert_eq!(ip.ip().to_string(), "::1");
}

#[test]
fn test_ip_network_contains() {
    let network: IpNetwork = "192.168.1.0/24".parse().unwrap();
    let host: IpNetwork = "192.168.1.100/32".parse().unwrap();
    assert!(network.contains(host.ip()));
}
