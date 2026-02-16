//! SNMP client implementation.

use anyhow::{Context, Result};
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::config::NetdiscoConfig;

/// SNMP protocol version.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SnmpVersion {
    V1,
    V2c,
    V3,
}

impl From<u8> for SnmpVersion {
    fn from(v: u8) -> Self {
        match v {
            1 => SnmpVersion::V1,
            2 => SnmpVersion::V2c,
            _ => SnmpVersion::V3,
        }
    }
}

/// SNMP authentication credentials.
#[derive(Debug, Clone)]
pub struct SnmpCredentials {
    pub version: SnmpVersion,
    pub community: String,
    // SNMPv3 fields
    pub username: Option<String>,
    pub auth_protocol: Option<String>,
    pub auth_password: Option<String>,
    pub priv_protocol: Option<String>,
    pub priv_password: Option<String>,
}

/// A high-level SNMP client for Netdisco operations.
#[derive(Debug)]
pub struct SnmpClient {
    target: SocketAddr,
    credentials: SnmpCredentials,
    timeout: Duration,
    retries: u32,
}

impl SnmpClient {
    /// Create a new SNMP client for the given device.
    pub fn new(
        host: &str,
        port: u16,
        credentials: SnmpCredentials,
        timeout_us: u64,
        retries: u32,
    ) -> Result<Self> {
        let target: SocketAddr = format!("{}:{}", host, port)
            .parse()
            .with_context(|| format!("Invalid SNMP target: {}:{}", host, port))?;

        Ok(Self {
            target,
            credentials,
            timeout: Duration::from_micros(timeout_us),
            retries,
        })
    }

    /// Create a client from Netdisco config for a specific device.
    pub fn from_config(config: &NetdiscoConfig, host: &str) -> Result<Self> {
        let community = config.community.first()
            .cloned()
            .unwrap_or_else(|| "public".to_string());

        Self::new(
            host,
            161,
            SnmpCredentials {
                version: SnmpVersion::from(config.snmpver),
                community,
                username: None,
                auth_protocol: None,
                auth_password: None,
                priv_protocol: None,
                priv_password: None,
            },
            config.snmptimeout,
            config.snmpretries,
        )
    }

    /// SNMP GET request for a single OID.
    pub fn get(&self, oid: &[u32]) -> Result<Vec<u8>> {
        debug!("SNMP GET {} from {}", oid_to_string(oid), self.target);
        // Build and send SNMPv2c GET PDU
        let pdu = build_get_pdu(&self.credentials.community, oid);
        let response = self.send_receive(&pdu)?;
        Ok(response)
    }

    /// SNMP GETNEXT request (walk single step).
    pub fn get_next(&self, oid: &[u32]) -> Result<(Vec<u32>, Vec<u8>)> {
        debug!("SNMP GETNEXT {} from {}", oid_to_string(oid), self.target);
        let pdu = build_getnext_pdu(&self.credentials.community, oid);
        let response = self.send_receive(&pdu)?;
        parse_getnext_response(&response)
    }

    /// SNMP GETBULK request (efficient table walking).
    pub fn get_bulk(&self, oid: &[u32], max_repetitions: u32) -> Result<Vec<(Vec<u32>, Vec<u8>)>> {
        debug!("SNMP GETBULK {} (max_rep={}) from {}", oid_to_string(oid), max_repetitions, self.target);
        let pdu = build_getbulk_pdu(&self.credentials.community, oid, max_repetitions);
        let response = self.send_receive(&pdu)?;
        parse_getbulk_response(&response)
    }

    /// Walk an entire OID subtree.
    pub fn walk(&self, base_oid: &[u32]) -> Result<Vec<(Vec<u32>, Vec<u8>)>> {
        info!("SNMP WALK {} on {}", oid_to_string(base_oid), self.target);
        let mut results = Vec::new();
        let mut current_oid = base_oid.to_vec();

        loop {
            match self.get_next(&current_oid) {
                Ok((next_oid, value)) => {
                    // Check if we've gone past the subtree
                    if !next_oid.starts_with(base_oid) {
                        break;
                    }
                    current_oid = next_oid.clone();
                    results.push((next_oid, value));
                }
                Err(e) => {
                    warn!("SNMP walk ended: {}", e);
                    break;
                }
            }
        }

        info!("SNMP WALK complete: {} results", results.len());
        Ok(results)
    }

    /// Get sysDescr, sysObjectID, sysUpTime, sysContact, sysName, sysLocation, sysServices.
    pub fn get_system_info(&self) -> Result<SystemInfo> {
        info!("Getting system info from {}", self.target);

        let sys_descr = self.get(&super::oids::SYS_DESCR).ok();
        let sys_object_id = self.get(&super::oids::SYS_OBJECT_ID).ok();
        let sys_uptime = self.get(&super::oids::SYS_UPTIME).ok();
        let sys_contact = self.get(&super::oids::SYS_CONTACT).ok();
        let sys_name = self.get(&super::oids::SYS_NAME).ok();
        let sys_location = self.get(&super::oids::SYS_LOCATION).ok();
        let sys_services = self.get(&super::oids::SYS_SERVICES).ok();

        Ok(SystemInfo {
            description: sys_descr.map(|v| String::from_utf8_lossy(&v).to_string()),
            object_id: sys_object_id.map(|v| String::from_utf8_lossy(&v).to_string()),
            uptime: sys_uptime.and_then(|v| parse_timeticks(&v)),
            contact: sys_contact.map(|v| String::from_utf8_lossy(&v).to_string()),
            name: sys_name.map(|v| String::from_utf8_lossy(&v).to_string()),
            location: sys_location.map(|v| String::from_utf8_lossy(&v).to_string()),
            services: sys_services.and_then(|v| parse_integer(&v)),
        })
    }

    /// Walk the ifTable to get interface information.
    pub fn get_interfaces(&self) -> Result<Vec<InterfaceInfo>> {
        info!("Getting interfaces from {}", self.target);
        let if_descr = self.walk(&super::oids::IF_DESCR)?;
        let if_type = self.walk(&super::oids::IF_TYPE)?;
        let if_speed = self.walk(&super::oids::IF_SPEED)?;
        let if_admin_status = self.walk(&super::oids::IF_ADMIN_STATUS)?;
        let if_oper_status = self.walk(&super::oids::IF_OPER_STATUS)?;

        let mut interfaces = Vec::new();
        for (oid, descr_bytes) in &if_descr {
            let ifindex = *oid.last().unwrap_or(&0) as i32;
            interfaces.push(InterfaceInfo {
                ifindex,
                descr: String::from_utf8_lossy(descr_bytes).to_string(),
                if_type: find_value_for_index(&if_type, ifindex)
                    .map(|v| String::from_utf8_lossy(&v).to_string()),
                speed: find_value_for_index(&if_speed, ifindex)
                    .and_then(|v| parse_integer(&v)),
                admin_status: find_value_for_index(&if_admin_status, ifindex)
                    .and_then(|v| parse_integer(&v)),
                oper_status: find_value_for_index(&if_oper_status, ifindex)
                    .and_then(|v| parse_integer(&v)),
            });
        }

        Ok(interfaces)
    }

    /// Walk the dot1dTpFdbTable (MAC address table) for macsuck.
    pub fn get_mac_table(&self) -> Result<Vec<MacEntry>> {
        info!("Getting MAC address table from {}", self.target);
        let fdb_port = self.walk(&super::oids::DOT1D_TP_FDB_PORT)?;
        let fdb_address = self.walk(&super::oids::DOT1D_TP_FDB_ADDRESS)?;

        let mut entries = Vec::new();
        for (oid, port_bytes) in &fdb_port {
            let bridge_port = parse_integer(port_bytes).unwrap_or(0);
            // MAC is encoded in the last 6 octets of the OID
            if oid.len() >= 6 {
                let mac_parts: Vec<String> = oid[oid.len()-6..]
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect();
                let mac = mac_parts.join(":");
                entries.push(MacEntry {
                    mac,
                    bridge_port: bridge_port as i32,
                    vlan: None,
                });
            }
        }

        Ok(entries)
    }

    /// Walk the ipNetToMediaTable (ARP table) for arpnip.
    pub fn get_arp_table(&self) -> Result<Vec<ArpEntry>> {
        info!("Getting ARP table from {}", self.target);
        let arp_phys = self.walk(&super::oids::IP_NET_TO_MEDIA_PHYS)?;

        let mut entries = Vec::new();
        for (oid, mac_bytes) in &arp_phys {
            // IP is encoded in the last 4 octets of the OID
            if oid.len() >= 4 {
                let ip = format!("{}.{}.{}.{}", oid[oid.len()-4], oid[oid.len()-3], oid[oid.len()-2], oid[oid.len()-1]);
                let mac = mac_bytes.iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<Vec<_>>()
                    .join(":");
                entries.push(ArpEntry { ip, mac });
            }
        }

        Ok(entries)
    }

    /// Send a PDU and receive the response with retry logic.
    fn send_receive(&self, pdu: &[u8]) -> Result<Vec<u8>> {
        let socket = UdpSocket::bind("0.0.0.0:0")
            .context("Failed to bind UDP socket")?;
        socket.set_read_timeout(Some(self.timeout))
            .context("Failed to set socket timeout")?;
        socket.connect(self.target)
            .context("Failed to connect to SNMP target")?;

        for attempt in 0..=self.retries {
            socket.send(pdu).context("Failed to send SNMP PDU")?;

            let mut buf = vec![0u8; 65535];
            match socket.recv(&mut buf) {
                Ok(len) => {
                    buf.truncate(len);
                    return Ok(buf);
                }
                Err(e) if attempt < self.retries => {
                    debug!("SNMP retry {} after error: {}", attempt + 1, e);
                    continue;
                }
                Err(e) => {
                    return Err(e).context("SNMP request timed out");
                }
            }
        }

        unreachable!()
    }
}

/// System information from SNMP system group.
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub description: Option<String>,
    pub object_id: Option<String>,
    pub uptime: Option<i64>,
    pub contact: Option<String>,
    pub name: Option<String>,
    pub location: Option<String>,
    pub services: Option<i64>,
}

/// Interface information from ifTable.
#[derive(Debug, Clone)]
pub struct InterfaceInfo {
    pub ifindex: i32,
    pub descr: String,
    pub if_type: Option<String>,
    pub speed: Option<i64>,
    pub admin_status: Option<i64>,
    pub oper_status: Option<i64>,
}

/// MAC address table entry.
#[derive(Debug, Clone)]
pub struct MacEntry {
    pub mac: String,
    pub bridge_port: i32,
    pub vlan: Option<i32>,
}

/// ARP table entry.
#[derive(Debug, Clone)]
pub struct ArpEntry {
    pub ip: String,
    pub mac: String,
}

// ==================== Helper Functions ====================

fn oid_to_string(oid: &[u32]) -> String {
    oid.iter().map(|o| o.to_string()).collect::<Vec<_>>().join(".")
}

fn find_value_for_index(table: &[(Vec<u32>, Vec<u8>)], index: i32) -> Option<Vec<u8>> {
    table.iter()
        .find(|(oid, _)| oid.last().copied() == Some(index as u32))
        .map(|(_, v)| v.clone())
}

fn parse_integer(data: &[u8]) -> Option<i64> {
    match data.len() {
        1 => Some(data[0] as i64),
        2 => Some(i16::from_be_bytes([data[0], data[1]]) as i64),
        4 => Some(i32::from_be_bytes([data[0], data[1], data[2], data[3]]) as i64),
        8 => Some(i64::from_be_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]])),
        _ => None,
    }
}

fn parse_timeticks(data: &[u8]) -> Option<i64> {
    parse_integer(data)
}

// Minimal SNMPv2c PDU builders (BER encoding)
fn build_get_pdu(community: &str, oid: &[u32]) -> Vec<u8> {
    build_pdu(0xa0, community, oid) // GetRequest-PDU
}

fn build_getnext_pdu(community: &str, oid: &[u32]) -> Vec<u8> {
    build_pdu(0xa1, community, oid) // GetNextRequest-PDU
}

fn build_getbulk_pdu(community: &str, oid: &[u32], max_repetitions: u32) -> Vec<u8> {
    // Simplified - real implementation would set non-repeaters and max-repetitions
    build_pdu(0xa5, community, oid) // GetBulkRequest-PDU
}

fn build_pdu(pdu_type: u8, community: &str, oid: &[u32]) -> Vec<u8> {
    let mut pdu = Vec::new();

    // Encode OID
    let mut oid_bytes = vec![0x06]; // OBJECT IDENTIFIER
    let mut oid_value = Vec::new();
    if oid.len() >= 2 {
        oid_value.push((oid[0] * 40 + oid[1]) as u8);
        for &component in &oid[2..] {
            if component < 128 {
                oid_value.push(component as u8);
            } else {
                let mut parts = Vec::new();
                let mut val = component;
                parts.push((val & 0x7f) as u8);
                val >>= 7;
                while val > 0 {
                    parts.push((val & 0x7f) as u8 | 0x80);
                    val >>= 7;
                }
                parts.reverse();
                oid_value.extend_from_slice(&parts);
            }
        }
    }
    oid_bytes.push(oid_value.len() as u8);
    oid_bytes.extend_from_slice(&oid_value);

    // VarBind: OID + NULL value
    let mut varbind = Vec::new();
    varbind.push(0x30); // SEQUENCE
    let varbind_content_len = oid_bytes.len() + 2; // OID + NULL (05 00)
    varbind.push(varbind_content_len as u8);
    varbind.extend_from_slice(&oid_bytes);
    varbind.extend_from_slice(&[0x05, 0x00]); // NULL

    // VarBindList
    let mut varbind_list = vec![0x30]; // SEQUENCE
    varbind_list.push(varbind.len() as u8);
    varbind_list.extend_from_slice(&varbind);

    // PDU
    let request_id: u32 = rand::random::<u16>() as u32;
    let mut pdu_content = Vec::new();
    // request-id
    pdu_content.extend_from_slice(&[0x02, 0x04]);
    pdu_content.extend_from_slice(&request_id.to_be_bytes());
    // error-status = 0
    pdu_content.extend_from_slice(&[0x02, 0x01, 0x00]);
    // error-index = 0
    pdu_content.extend_from_slice(&[0x02, 0x01, 0x00]);
    pdu_content.extend_from_slice(&varbind_list);

    let mut pdu_bytes = vec![pdu_type];
    encode_length(&mut pdu_bytes, pdu_content.len());
    pdu_bytes.extend_from_slice(&pdu_content);

    // Community string
    let mut comm_bytes = vec![0x04]; // OCTET STRING
    comm_bytes.push(community.len() as u8);
    comm_bytes.extend_from_slice(community.as_bytes());

    // Message: version + community + PDU
    let mut message_content = Vec::new();
    // Version: SNMPv2c = 1
    message_content.extend_from_slice(&[0x02, 0x01, 0x01]);
    message_content.extend_from_slice(&comm_bytes);
    message_content.extend_from_slice(&pdu_bytes);

    // Wrap in SEQUENCE
    pdu.push(0x30); // SEQUENCE
    encode_length(&mut pdu, message_content.len());
    pdu.extend_from_slice(&message_content);

    pdu
}

fn encode_length(buf: &mut Vec<u8>, len: usize) {
    if len < 128 {
        buf.push(len as u8);
    } else if len < 256 {
        buf.push(0x81);
        buf.push(len as u8);
    } else {
        buf.push(0x82);
        buf.push((len >> 8) as u8);
        buf.push((len & 0xff) as u8);
    }
}

fn parse_getnext_response(data: &[u8]) -> Result<(Vec<u32>, Vec<u8>)> {
    // Simplified BER parser - extract first varbind OID and value
    // In production, use a proper ASN.1/BER parser
    Ok((vec![], data.to_vec()))
}

fn parse_getbulk_response(data: &[u8]) -> Result<Vec<(Vec<u32>, Vec<u8>)>> {
    Ok(vec![])
}
