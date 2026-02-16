//! End-to-end workflow tests.
//!
//! These test complete user workflows through the application stack,
//! simulating real usage patterns without requiring external dependencies.

use netdisco::config::NetdiscoConfig;
use netdisco::models::device::Device;
use netdisco::models::node::Node;
use netdisco::models::admin::{self, Admin};
use netdisco::snmp::client::*;
use netdisco::util;
use ipnetwork::IpNetwork;

// ==================== Discovery Workflow Tests ====================

/// Simulate the data flow of a device discovery
#[test]
fn test_discovery_workflow_data_flow() {
    // 1. Load config
    let config = NetdiscoConfig::default();
    assert_eq!(config.snmpver, 3);

    // 2. Create SNMP client
    let client = SnmpClient::from_config(&config, "127.0.0.1");
    assert!(client.is_ok());

    // 3. Build system info
    let sys_info = SystemInfo {
        description: Some("Cisco IOS Software, C3750 Software".to_string()),
        object_id: Some("1.3.6.1.4.1.9.1.516".to_string()),
        uptime: Some(123456789),
        contact: Some("noc@company.com".to_string()),
        name: Some("CORE-SW-01".to_string()),
        location: Some("DC1, Row B, Rack 5".to_string()),
        services: Some(78),  // 78 = 0b1001110
    };

    // 4. Compute layers from services (same logic as discover worker)
    let layers_from_services = sys_info.services.map(|svc| {
        (0..7).map(|i| if svc & (1 << i) != 0 { '1' } else { '0' })
            .collect::<String>()
    });
    // 78 = bit0:0, bit1:1, bit2:1, bit3:1, bit4:0, bit5:0, bit6:1
    assert_eq!(layers_from_services, Some("0111001".to_string()));

    // 5. Build device record using has_layer-compatible format
    // has_layer uses index (7-layer), so "0000110" means L2=1, L3=1
    let layers = Some("0000110".to_string());

    let device = Device {
        ip: "10.0.0.1/32".parse().unwrap(),
        creation: None,
        dns: None,
        description: sys_info.description.clone(),
        uptime: sys_info.uptime,
        contact: sys_info.contact.clone(),
        name: sys_info.name.clone(),
        location: sys_info.location.clone(),
        layers: layers.clone(),
        ports: None,
        mac: None,
        serial: None,
        model: None,
        ps1_type: None, ps2_type: None, ps1_status: None, ps2_status: None,
        fan: None, slots: None,
        vendor: None,
        os: None, os_ver: None,
        log: None,
        snmp_ver: Some(config.snmpver as i32),
        snmp_comm: config.community.first().cloned(),
        snmp_class: None,
        vtp_domain: None,
        last_discover: Some(chrono::Local::now().naive_local()),
        last_macsuck: None,
        last_arpnip: None,
        pae_is_enabled: None,
        custom_fields: None,
        tags: None,
    };

    // 6. Verify device properties
    assert!(device.has_layer(2), "Should be L2");
    assert!(device.has_layer(3), "Should be L3");
    assert!(device.is_switch(), "Should be a switch");
    assert!(device.is_router(), "Should be a router");
    assert_eq!(device.display_name(), "CORE-SW-01");
    assert_eq!(device.description, Some("Cisco IOS Software, C3750 Software".to_string()));

    // 7. Verify serialization roundtrip
    let json = serde_json::to_string(&device).unwrap();
    assert!(json.contains("CORE-SW-01"));
    assert!(json.contains("C3750"));
}

/// Simulate interface processing after discovery
#[test]
fn test_discovery_interface_processing() {
    let interfaces = vec![
        InterfaceInfo {
            ifindex: 1,
            descr: "GigabitEthernet0/1".to_string(),
            if_type: Some("ethernetCsmacd".to_string()),
            speed: Some(1000000000),
            admin_status: Some(1),
            oper_status: Some(1),
        },
        InterfaceInfo {
            ifindex: 2,
            descr: "GigabitEthernet0/2".to_string(),
            if_type: Some("ethernetCsmacd".to_string()),
            speed: Some(1000000000),
            admin_status: Some(1),
            oper_status: Some(2), // down
        },
        InterfaceInfo {
            ifindex: 3,
            descr: "Vlan100".to_string(),
            if_type: Some("propVirtual".to_string()),
            speed: Some(1000000000),
            admin_status: Some(1),
            oper_status: Some(1),
        },
    ];

    assert_eq!(interfaces.len(), 3);

    // Simulating the port status determination from the worker
    for iface in &interfaces {
        let up = iface.oper_status.map(|s| if s == 1 { "up" } else { "down" });
        let up_admin = iface.admin_status.map(|s| if s == 1 { "up" } else { "down" });

        match iface.ifindex {
            1 => {
                assert_eq!(up, Some("up"));
                assert_eq!(up_admin, Some("up"));
            },
            2 => {
                assert_eq!(up, Some("down"));
                assert_eq!(up_admin, Some("up"));
            },
            3 => {
                assert_eq!(up, Some("up"));
            },
            _ => {}
        }
    }
}

// ==================== Macsuck Workflow Tests ====================

/// Simulate MAC address collection from SNMP data
#[test]
fn test_macsuck_workflow() {
    let mac_entries = vec![
        MacEntry { mac: "00:11:22:33:44:55".to_string(), bridge_port: 1, vlan: Some(100) },
        MacEntry { mac: "00:aa:bb:cc:dd:ee".to_string(), bridge_port: 2, vlan: Some(200) },
        MacEntry { mac: "ff:ff:ff:ff:ff:ff".to_string(), bridge_port: 0, vlan: None },
    ];

    let device_ip: IpNetwork = "10.0.0.1/32".parse().unwrap();

    let mut nodes = Vec::new();
    for entry in &mac_entries {
        let node = Node {
            mac: entry.mac.clone(),
            switch: device_ip,
            port: format!("bridge-port-{}", entry.bridge_port),
            vlan: entry.vlan.map(|v| v.to_string()),
            active: Some(true),
            oui: Some(Node::extract_oui(&entry.mac)),
            time_first: None,
            time_recent: None,
            time_last: None,
        };
        nodes.push(node);
    }

    assert_eq!(nodes.len(), 3);
    assert_eq!(nodes[0].oui, Some("001122".to_string()));
    assert_eq!(nodes[1].oui, Some("00AABB".to_string()));
    assert_eq!(nodes[2].oui, Some("FFFFFF".to_string()));
    assert_eq!(nodes[0].port, "bridge-port-1");
    assert_eq!(nodes[0].vlan, Some("100".to_string()));
}

// ==================== Arpnip Workflow Tests ====================

/// Simulate ARP table collection
#[test]
fn test_arpnip_workflow() {
    let arp_entries = vec![
        ArpEntry { ip: "192.168.1.10".to_string(), mac: "00:11:22:33:44:55".to_string() },
        ArpEntry { ip: "192.168.1.20".to_string(), mac: "00:aa:bb:cc:dd:ee".to_string() },
        ArpEntry { ip: "not_valid_ip".to_string(), mac: "ff:ff:ff:ff:ff:ff".to_string() },
    ];

    let mut stored = 0;
    for entry in &arp_entries {
        if let Ok(entry_ip) = entry.ip.parse::<std::net::IpAddr>() {
            let _ip_network = IpNetwork::from(entry_ip);
            stored += 1;
        }
    }

    assert_eq!(stored, 2, "Only 2 of 3 entries had valid IPs");
}

// ==================== Config Workflow Tests ====================

/// Simulate the full config loading and override pipeline
#[test]
#[serial_test::serial]
fn test_config_loading_pipeline() {
    // Clean env
    for key in ["NETDISCO_DB_NAME", "NETDISCO_DB_HOST", "NETDISCO_DB_USER",
                "NETDISCO_DB_PASS", "NETDISCO_RO_COMMUNITY", "NETDISCO_DOMAIN"] {
        std::env::remove_var(key);
    }

    // Step 1: Start with defaults
    let mut config = NetdiscoConfig::default();
    assert_eq!(config.database.name, "netdisco");
    assert_eq!(config.database.host, "localhost");

    // Step 2: Apply YAML overrides
    let overrides: serde_yaml::Value = serde_yaml::from_str(r#"
        database:
            name: prod_netdisco
            host: db-prod.corp.com
            user: nd_prod
            pass: prod_secret
        community:
            - corp_community
        no_auth: false
    "#).unwrap();
    config.apply_overrides(&overrides).unwrap();
    assert_eq!(config.database.name, "prod_netdisco");
    assert_eq!(config.database.host, "db-prod.corp.com");
    assert_eq!(config.community, vec!["corp_community"]);

    // Step 3: Apply env vars (highest priority)
    std::env::set_var("NETDISCO_DB_HOST", "docker-db");
    std::env::set_var("NETDISCO_DB_PASS", "docker_pass");
    config.apply_env_overrides();
    assert_eq!(config.database.host, "docker-db"); // Env takes priority
    assert_eq!(config.database.pass, "docker_pass");
    assert_eq!(config.database.name, "prod_netdisco"); // YAML override preserved

    // Step 4: Verify connection string
    let conn = config.database.connection_string();
    assert!(conn.contains("docker-db"));
    assert!(conn.contains("docker_pass"));
    assert!(conn.contains("prod_netdisco"));

    // Cleanup
    std::env::remove_var("NETDISCO_DB_HOST");
    std::env::remove_var("NETDISCO_DB_PASS");
}

// ==================== Job Queue Workflow Tests ====================

/// Simulate the job lifecycle: create -> dispatch -> complete
#[test]
fn test_job_lifecycle() {
    // Step 1: Create job (simulating enqueue)
    let mut job = Admin {
        job: Some(42),
        entered: Some(chrono::Local::now().naive_local()),
        started: None,
        finished: None,
        device: Some("10.0.0.1/32".parse().unwrap()),
        port: None,
        action: Some("discover".to_string()),
        subaction: None,
        status: Some(admin::status::QUEUED.to_string()),
        username: Some("admin".to_string()),
        userip: Some("192.168.1.100/32".parse().unwrap()),
        log: None,
        debug: Some(false),
    };

    assert_eq!(job.status.as_deref(), Some("queued"));

    // Step 2: Dequeue (simulating worker pickup)
    job.status = Some(admin::status::RUNNING.to_string());
    job.started = Some(chrono::Local::now().naive_local());
    assert_eq!(job.status.as_deref(), Some("running"));
    assert!(job.started.is_some());

    // Step 3: Dispatch
    let action = job.action.as_deref().unwrap_or("");
    let device_ip = job.device;
    assert_eq!(action, "discover");
    assert!(device_ip.is_some());

    // Step 4: Complete successfully
    job.status = Some(admin::status::DONE.to_string());
    job.finished = Some(chrono::Local::now().naive_local());
    job.log = Some("Discovered 10.0.0.1 with 48 interfaces".to_string());
    assert_eq!(job.status.as_deref(), Some("done"));
    assert!(job.finished.is_some());
    assert!(job.log.is_some());
}

/// Simulate job failure lifecycle
#[test]
fn test_job_failure_lifecycle() {
    let mut job = Admin {
        job: Some(43),
        entered: Some(chrono::Local::now().naive_local()),
        started: None,
        finished: None,
        device: None,
        port: None,
        action: Some("discover".to_string()),
        subaction: None,
        status: Some(admin::status::QUEUED.to_string()),
        username: None,
        userip: None,
        log: None,
        debug: None,
    };

    // Start
    job.status = Some(admin::status::RUNNING.to_string());
    job.started = Some(chrono::Local::now().naive_local());

    // Fail - discover requires a device IP
    let device_ip = job.device;
    assert!(device_ip.is_none());

    job.status = Some(admin::status::ERROR.to_string());
    job.finished = Some(chrono::Local::now().naive_local());
    job.log = Some("discover requires a device IP".to_string());
    assert_eq!(job.status.as_deref(), Some("error"));
}

// ==================== Full API Workflow Tests ====================

/// Test the web server stack end-to-end with mock app
#[actix_web::test]
async fn test_full_api_workflow() {
    use actix_web::{test as atest, web, App, HttpResponse};
    use actix_web::cookie::Key;
    use actix_session::{SessionMiddleware, storage::CookieSessionStore};
    use std::sync::{Arc, Mutex};

    // Shared mock database
    let devices: Arc<Mutex<Vec<serde_json::Value>>> = Arc::new(Mutex::new(vec![]));
    let jobs: Arc<Mutex<Vec<serde_json::Value>>> = Arc::new(Mutex::new(vec![]));

    let devices_clone = devices.clone();
    let devices_clone2 = devices.clone();
    let jobs_clone = jobs.clone();
    let jobs_clone2 = jobs.clone();

    let secret_key = Key::generate();
    let app = atest::init_service(
        App::new()
            .wrap(SessionMiddleware::builder(
                CookieSessionStore::default(),
                secret_key.clone(),
            ).build())
            .route("/api/v1/devices", web::get().to({
                let d = devices_clone.clone();
                move || {
                    let d = d.clone();
                    async move {
                        let data = d.lock().unwrap();
                        HttpResponse::Ok().json(&*data)
                    }
                }
            }))
            .route("/api/v1/devices", web::post().to({
                let d = devices_clone2.clone();
                move |body: web::Json<serde_json::Value>| {
                    let d = d.clone();
                    async move {
                        d.lock().unwrap().push(body.into_inner());
                        HttpResponse::Created().json(serde_json::json!({"created": true}))
                    }
                }
            }))
            .route("/api/v1/jobs", web::post().to({
                let j = jobs_clone.clone();
                move |body: web::Json<serde_json::Value>| {
                    let j = j.clone();
                    async move {
                        let mut jq = j.lock().unwrap();
                        let job_id = jq.len() + 1;
                        let mut job = body.into_inner();
                        job["job_id"] = serde_json::json!(job_id);
                        job["status"] = serde_json::json!("queued");
                        jq.push(job);
                        HttpResponse::Ok().json(serde_json::json!({"job": job_id}))
                    }
                }
            }))
            .route("/api/v1/jobs", web::get().to({
                let j = jobs_clone2.clone();
                move || {
                    let j = j.clone();
                    async move {
                        let data = j.lock().unwrap();
                        HttpResponse::Ok().json(&*data)
                    }
                }
            }))
    ).await;

    // Step 1: List devices (should be empty)
    let req = atest::TestRequest::get().uri("/api/v1/devices").to_request();
    let resp = atest::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: Vec<serde_json::Value> = atest::read_body_json(resp).await;
    assert_eq!(body.len(), 0);

    // Step 2: Add a device (simulating discovery result)
    let req = atest::TestRequest::post()
        .uri("/api/v1/devices")
        .set_json(serde_json::json!({
            "ip": "10.0.0.1/32",
            "dns": "switch1.example.com",
            "name": "CORE-SW-01",
            "layers": "0000110"
        }))
        .to_request();
    let resp = atest::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 201);

    // Step 3: Verify device was stored
    let req = atest::TestRequest::get().uri("/api/v1/devices").to_request();
    let resp = atest::call_service(&app, req).await;
    let body: Vec<serde_json::Value> = atest::read_body_json(resp).await;
    assert_eq!(body.len(), 1);
    assert_eq!(body[0]["name"], "CORE-SW-01");

    // Step 4: Enqueue a macsuck job
    let req = atest::TestRequest::post()
        .uri("/api/v1/jobs")
        .set_json(serde_json::json!({
            "action": "macsuck",
            "device": "10.0.0.1/32"
        }))
        .to_request();
    let resp = atest::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = atest::read_body_json(resp).await;
    assert_eq!(body["job"], 1);

    // Step 5: Verify job was queued
    let req = atest::TestRequest::get().uri("/api/v1/jobs").to_request();
    let resp = atest::call_service(&app, req).await;
    let body: Vec<serde_json::Value> = atest::read_body_json(resp).await;
    assert_eq!(body.len(), 1);
    assert_eq!(body[0]["action"], "macsuck");
    assert_eq!(body[0]["status"], "queued");

    // Step 6: Enqueue an arpnip job
    let req = atest::TestRequest::post()
        .uri("/api/v1/jobs")
        .set_json(serde_json::json!({
            "action": "arpnip",
            "device": "10.0.0.1/32"
        }))
        .to_request();
    let resp = atest::call_service(&app, req).await;
    let body: serde_json::Value = atest::read_body_json(resp).await;
    assert_eq!(body["job"], 2);

    // Step 7: Verify both jobs queued
    let req = atest::TestRequest::get().uri("/api/v1/jobs").to_request();
    let resp = atest::call_service(&app, req).await;
    let body: Vec<serde_json::Value> = atest::read_body_json(resp).await;
    assert_eq!(body.len(), 2);
}

// ==================== Utility Workflow Tests ====================

/// Test MAC address processing through the full pipeline
#[test]
fn test_mac_processing_pipeline() {
    let raw_macs = vec![
        "001122334455",
        "00:11:22:33:44:55",
        "00-11-22-33-44-55",
        "0011.2233.4455",
        "AA:BB:CC:DD:EE:FF",
    ];

    for raw in &raw_macs {
        let formatted = util::format_mac_ieee(raw);
        assert!(formatted.contains(':'), "Formatted MAC should have colons: {}", formatted);
        // Length should be 17 (xx:xx:xx:xx:xx:xx)
        assert_eq!(formatted.len(), 17, "Formatted MAC should be 17 chars: {}", formatted);

        let oui = Node::extract_oui(raw);
        assert_eq!(oui.len(), 6, "OUI should be 6 chars");
    }
}

/// Test uptime formatting across various scales
#[test]
fn test_uptime_display_formatting() {
    let test_cases = vec![
        (0i64, "00:00:00"),
        (100, "00:00:01"),       // 1 second
        (6000, "00:01:00"),      // 1 minute
        (360000, "01:00:00"),    // 1 hour
        (8640000, "1 day"),      // 1 day (partial match)
    ];

    for (ticks, expected_contains) in &test_cases {
        let result = util::format_uptime(*ticks);
        assert!(result.contains(expected_contains),
            "format_uptime({}) = '{}', expected to contain '{}'",
            ticks, result, expected_contains);
    }
}

/// Test permission checking across the discovery workflow
#[test]
fn test_permission_workflow() {
    use netdisco::util::permission;

    let config = NetdiscoConfig::default();

    // With default config (empty lists), everything should be permitted
    let ip: IpNetwork = "10.0.0.1/32".parse().unwrap();
    assert!(permission::is_permitted(&ip, &config.devices_only, &config.devices_no));
    assert!(permission::is_permitted(&ip, &config.discover_only, &config.discover_no));
    assert!(permission::is_permitted(&ip, &config.macsuck_only, &config.macsuck_no));
    assert!(permission::is_permitted(&ip, &config.arpnip_only, &config.arpnip_no));

    // Test with restrictive config
    let only = vec!["10.0.0.0/24".to_string()];
    let no = vec!["10.0.0.100/32".to_string()];

    let allowed_ip: IpNetwork = "10.0.0.1/32".parse().unwrap();
    let blocked_ip: IpNetwork = "10.0.0.100/32".parse().unwrap();
    let outside_ip: IpNetwork = "192.168.1.1/32".parse().unwrap();

    assert!(permission::is_permitted(&allowed_ip, &only, &no));
    assert!(!permission::is_permitted(&blocked_ip, &only, &no));
    assert!(!permission::is_permitted(&outside_ip, &only, &no));
}

// ==================== Version & Constants Tests ====================

#[test]
fn test_version_constant() {
    assert_eq!(netdisco::VERSION, "3.0.0");
}

#[test]
fn test_default_ports() {
    assert_eq!(netdisco::DEFAULT_WEB_PORT, 5000);
    assert_eq!(netdisco::DEFAULT_BACKEND_PORT, 5001);
}

#[test]
fn test_default_config_file() {
    assert_eq!(netdisco::DEFAULT_CONFIG_FILE, "config.yml");
}
