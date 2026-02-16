//! Unit tests for SNMP client and OID definitions.

use netdisco::snmp::client::*;
use netdisco::snmp::oids;
use netdisco::config::NetdiscoConfig;

// ==================== SnmpVersion Tests ====================

#[test]
fn test_snmp_version_from_1() {
    assert_eq!(SnmpVersion::from(1), SnmpVersion::V1);
}

#[test]
fn test_snmp_version_from_2() {
    assert_eq!(SnmpVersion::from(2), SnmpVersion::V2c);
}

#[test]
fn test_snmp_version_from_3() {
    assert_eq!(SnmpVersion::from(3), SnmpVersion::V3);
}

#[test]
fn test_snmp_version_from_other() {
    // Any unrecognized version defaults to V3
    assert_eq!(SnmpVersion::from(0), SnmpVersion::V3);
    assert_eq!(SnmpVersion::from(4), SnmpVersion::V3);
    assert_eq!(SnmpVersion::from(255), SnmpVersion::V3);
}

// ==================== SnmpCredentials Tests ====================

#[test]
fn test_snmp_credentials_v2c() {
    let creds = SnmpCredentials {
        version: SnmpVersion::V2c,
        community: "public".to_string(),
        username: None,
        auth_protocol: None,
        auth_password: None,
        priv_protocol: None,
        priv_password: None,
    };
    assert_eq!(creds.version, SnmpVersion::V2c);
    assert_eq!(creds.community, "public");
}

#[test]
fn test_snmp_credentials_v3() {
    let creds = SnmpCredentials {
        version: SnmpVersion::V3,
        community: "".to_string(),
        username: Some("snmpuser".to_string()),
        auth_protocol: Some("SHA".to_string()),
        auth_password: Some("authpass".to_string()),
        priv_protocol: Some("AES".to_string()),
        priv_password: Some("privpass".to_string()),
    };
    assert_eq!(creds.version, SnmpVersion::V3);
    assert_eq!(creds.username, Some("snmpuser".to_string()));
}

// ==================== SnmpClient Creation Tests ====================

#[test]
fn test_snmp_client_new_valid_ipv4() {
    let creds = SnmpCredentials {
        version: SnmpVersion::V2c,
        community: "public".to_string(),
        username: None,
        auth_protocol: None,
        auth_password: None,
        priv_protocol: None,
        priv_password: None,
    };
    let client = SnmpClient::new("127.0.0.1", 161, creds, 3_000_000, 2);
    assert!(client.is_ok());
}

#[test]
fn test_snmp_client_new_custom_port() {
    let creds = SnmpCredentials {
        version: SnmpVersion::V2c,
        community: "public".to_string(),
        username: None,
        auth_protocol: None,
        auth_password: None,
        priv_protocol: None,
        priv_password: None,
    };
    let client = SnmpClient::new("10.0.0.1", 1161, creds, 1_000_000, 1);
    assert!(client.is_ok());
}

#[test]
fn test_snmp_client_new_invalid_host() {
    let creds = SnmpCredentials {
        version: SnmpVersion::V2c,
        community: "public".to_string(),
        username: None,
        auth_protocol: None,
        auth_password: None,
        priv_protocol: None,
        priv_password: None,
    };
    let client = SnmpClient::new("not_a_valid_ip_address", 161, creds, 3_000_000, 2);
    assert!(client.is_err());
}

#[test]
fn test_snmp_client_from_config_default_community() {
    let config = NetdiscoConfig::default();
    let client = SnmpClient::from_config(&config, "127.0.0.1");
    assert!(client.is_ok());
}

#[test]
fn test_snmp_client_from_config_custom_community() {
    let mut config = NetdiscoConfig::default();
    config.community = vec!["secret_comm".to_string()];
    let client = SnmpClient::from_config(&config, "10.0.0.1");
    assert!(client.is_ok());
}

// ==================== SystemInfo Tests ====================

#[test]
fn test_system_info_struct() {
    let info = SystemInfo {
        description: Some("Cisco IOS XE Software".to_string()),
        object_id: Some("1.3.6.1.4.1.9.1.2066".to_string()),
        uptime: Some(123456789),
        contact: Some("admin@example.com".to_string()),
        name: Some("CORE-SW-01".to_string()),
        location: Some("DC1, Rack 5, Unit 20".to_string()),
        services: Some(78),
    };
    assert_eq!(info.name, Some("CORE-SW-01".to_string()));
    assert_eq!(info.uptime, Some(123456789));
}

#[test]
fn test_system_info_all_none() {
    let info = SystemInfo {
        description: None,
        object_id: None,
        uptime: None,
        contact: None,
        name: None,
        location: None,
        services: None,
    };
    assert!(info.description.is_none());
    assert!(info.name.is_none());
}

// ==================== InterfaceInfo Tests ====================

#[test]
fn test_interface_info_struct() {
    let iface = InterfaceInfo {
        ifindex: 1,
        descr: "GigabitEthernet0/1".to_string(),
        if_type: Some("ethernetCsmacd".to_string()),
        speed: Some(1000000000),
        admin_status: Some(1),
        oper_status: Some(1),
    };
    assert_eq!(iface.ifindex, 1);
    assert_eq!(iface.descr, "GigabitEthernet0/1");
    assert_eq!(iface.speed, Some(1000000000));
}

// ==================== MacEntry/ArpEntry Tests ====================

#[test]
fn test_mac_entry_struct() {
    let entry = MacEntry {
        mac: "00:11:22:33:44:55".to_string(),
        bridge_port: 3,
        vlan: Some(100),
    };
    assert_eq!(entry.mac, "00:11:22:33:44:55");
    assert_eq!(entry.bridge_port, 3);
    assert_eq!(entry.vlan, Some(100));
}

#[test]
fn test_arp_entry_struct() {
    let entry = ArpEntry {
        ip: "192.168.1.100".to_string(),
        mac: "00:11:22:33:44:55".to_string(),
    };
    assert_eq!(entry.ip, "192.168.1.100");
    assert_eq!(entry.mac, "00:11:22:33:44:55");
}

// ==================== OID Constants Tests ====================

#[test]
fn test_oid_sys_descr() {
    assert_eq!(oids::SYS_DESCR, [1, 3, 6, 1, 2, 1, 1, 1]);
}

#[test]
fn test_oid_sys_object_id() {
    assert_eq!(oids::SYS_OBJECT_ID, [1, 3, 6, 1, 2, 1, 1, 2]);
}

#[test]
fn test_oid_sys_uptime() {
    assert_eq!(oids::SYS_UPTIME, [1, 3, 6, 1, 2, 1, 1, 3]);
}

#[test]
fn test_oid_sys_name() {
    assert_eq!(oids::SYS_NAME, [1, 3, 6, 1, 2, 1, 1, 5]);
}

#[test]
fn test_oid_if_descr() {
    assert_eq!(oids::IF_DESCR, [1, 3, 6, 1, 2, 1, 2, 2, 1, 2]);
}

#[test]
fn test_oid_dot1d_tp_fdb_port() {
    assert_eq!(oids::DOT1D_TP_FDB_PORT, [1, 3, 6, 1, 2, 1, 17, 4, 3, 1, 2]);
}

#[test]
fn test_oid_ip_net_to_media_phys() {
    assert_eq!(oids::IP_NET_TO_MEDIA_PHYS, [1, 3, 6, 1, 2, 1, 4, 22, 1, 2]);
}

#[test]
fn test_oid_lldp_rem_sys_name() {
    assert_eq!(oids::LLDP_REM_SYS_NAME, [1, 0, 8802, 1, 1, 2, 1, 4, 1, 9]);
}

#[test]
fn test_oid_pse_port_power() {
    assert_eq!(oids::PSE_PORT_POWER, [1, 3, 6, 1, 2, 1, 105, 1, 1, 1, 7]);
}

#[test]
fn test_oid_system_group_contiguous() {
    // Verify system group OIDs are contiguous 1.3.6.1.2.1.1.X
    let oids_list = [
        &oids::SYS_DESCR,
        &oids::SYS_OBJECT_ID,
        &oids::SYS_UPTIME,
        &oids::SYS_CONTACT,
        &oids::SYS_NAME,
        &oids::SYS_LOCATION,
        &oids::SYS_SERVICES,
    ];
    for (i, oid) in oids_list.iter().enumerate() {
        assert_eq!(oid[7], (i + 1) as u32, "System OID index mismatch at {}", i);
        // Verify common prefix
        assert_eq!(oid[0..7], [1, 3, 6, 1, 2, 1, 1]);
    }
}

#[test]
fn test_oid_if_table_contiguous() {
    // Verify ifTable OIDs share 1.3.6.1.2.1.2.2.1.X prefix
    let oids_list = [
        (&oids::IF_INDEX, 1u32),
        (&oids::IF_DESCR, 2),
        (&oids::IF_TYPE, 3),
        (&oids::IF_MTU, 4),
        (&oids::IF_SPEED, 5),
        (&oids::IF_PHYS_ADDRESS, 6),
        (&oids::IF_ADMIN_STATUS, 7),
        (&oids::IF_OPER_STATUS, 8),
        (&oids::IF_LAST_CHANGE, 9),
    ];
    for (oid, expected_idx) in &oids_list {
        assert_eq!(oid[0..9], [1, 3, 6, 1, 2, 1, 2, 2, 1]);
        assert_eq!(oid[9], *expected_idx);
    }
}
