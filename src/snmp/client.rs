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

impl SnmpVersion {
    /// BER-encoded version number for the SNMP message header.
    fn version_byte(&self) -> u8 {
        match self {
            SnmpVersion::V1 => 0,
            SnmpVersion::V2c => 1,
            SnmpVersion::V3 => 3,
        }
    }
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
        if credentials.version == SnmpVersion::V3 {
            warn!("SNMPv3 is not yet implemented, falling back to v2c for {}", host);
        }

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

    /// Effective SNMP version (v3 falls back to v2c until implemented).
    fn effective_version(&self) -> &SnmpVersion {
        if self.credentials.version == SnmpVersion::V3 {
            &SnmpVersion::V2c
        } else {
            &self.credentials.version
        }
    }

    /// SNMP GET request for a single OID.
    pub fn get(&self, oid: &[u32]) -> Result<Vec<u8>> {
        debug!("SNMP GET {} from {}", oid_to_string(oid), self.target);
        let pdu = build_get_pdu(self.effective_version(), &self.credentials.community, oid);
        let response = self.send_receive(&pdu)?;
        parse_get_response(&response)
    }

    /// SNMP GETNEXT request (walk single step).
    pub fn get_next(&self, oid: &[u32]) -> Result<(Vec<u32>, Vec<u8>)> {
        debug!("SNMP GETNEXT {} from {}", oid_to_string(oid), self.target);
        let pdu = build_getnext_pdu(self.effective_version(), &self.credentials.community, oid);
        let response = self.send_receive(&pdu)?;
        parse_getnext_response(&response)
    }

    /// SNMP GETBULK request (efficient table walking).
    pub fn get_bulk(&self, oid: &[u32], max_repetitions: u32) -> Result<Vec<(Vec<u32>, Vec<u8>)>> {
        debug!("SNMP GETBULK {} (max_rep={}) from {}", oid_to_string(oid), max_repetitions, self.target);
        let pdu = build_getbulk_pdu(self.effective_version(), &self.credentials.community, oid, max_repetitions);
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
                    if next_oid.is_empty() || !next_oid.starts_with(base_oid) {
                        break;
                    }
                    // Guard against infinite loops (OID didn't advance)
                    if next_oid <= current_oid {
                        warn!("SNMP walk: OID did not advance, stopping");
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
        let _fdb_address = self.walk(&super::oids::DOT1D_TP_FDB_ADDRESS)?;

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

// ==================== BER Parsing Helpers ====================

/// Read a BER tag byte (returns the tag).
fn read_tag(data: &[u8], pos: &mut usize) -> Result<u8> {
    if *pos >= data.len() {
        anyhow::bail!("BER: unexpected end of data reading tag");
    }
    let tag = data[*pos];
    *pos += 1;
    Ok(tag)
}

/// Read a BER length field and return the length value.
fn read_length(data: &[u8], pos: &mut usize) -> Result<usize> {
    if *pos >= data.len() {
        anyhow::bail!("BER: unexpected end of data reading length");
    }
    let first = data[*pos];
    *pos += 1;

    if first < 128 {
        Ok(first as usize)
    } else {
        let num_bytes = (first & 0x7f) as usize;
        if num_bytes > 4 || *pos + num_bytes > data.len() {
            anyhow::bail!("BER: invalid length encoding");
        }
        let mut len: usize = 0;
        for i in 0..num_bytes {
            len = (len << 8) | data[*pos + i] as usize;
        }
        *pos += num_bytes;
        Ok(len)
    }
}

/// Skip a BER TLV (tag + length + value), returning the bytes skipped.
fn skip_tlv(data: &[u8], pos: &mut usize) -> Result<()> {
    read_tag(data, pos)?;
    let len = read_length(data, pos)?;
    if *pos + len > data.len() {
        anyhow::bail!("BER: value extends past end of data");
    }
    *pos += len;
    Ok(())
}

/// Read the content bytes of a TLV without interpreting the tag.
fn read_tlv_value(data: &[u8], pos: &mut usize) -> Result<(u8, Vec<u8>)> {
    let tag = read_tag(data, pos)?;
    let len = read_length(data, pos)?;
    if *pos + len > data.len() {
        anyhow::bail!("BER: value extends past end of data");
    }
    let value = data[*pos..*pos + len].to_vec();
    *pos += len;
    Ok((tag, value))
}

/// Decode an OID from BER-encoded bytes.
fn decode_oid(data: &[u8]) -> Result<Vec<u32>> {
    if data.is_empty() {
        anyhow::bail!("BER: empty OID");
    }
    let mut oid = Vec::new();
    oid.push((data[0] / 40) as u32);
    oid.push((data[0] % 40) as u32);

    let mut i = 1;
    while i < data.len() {
        let mut component: u32 = 0;
        loop {
            if i >= data.len() {
                anyhow::bail!("BER: truncated OID component");
            }
            let byte = data[i];
            i += 1;
            component = (component << 7) | (byte & 0x7f) as u32;
            if byte & 0x80 == 0 {
                break;
            }
        }
        oid.push(component);
    }
    Ok(oid)
}

/// Parse an SNMP response and extract the first varbind (OID, value).
fn parse_snmp_response(data: &[u8]) -> Result<Vec<(Vec<u32>, Vec<u8>)>> {
    if data.len() < 2 {
        anyhow::bail!("SNMP response too short");
    }

    let mut pos: usize = 0;

    // Outer SEQUENCE
    let tag = read_tag(data, &mut pos)?;
    if tag != 0x30 {
        anyhow::bail!("SNMP: expected SEQUENCE (0x30), got 0x{:02x}", tag);
    }
    let _msg_len = read_length(data, &mut pos)?;

    // Version (INTEGER)
    skip_tlv(data, &mut pos)?;

    // Community (OCTET STRING)
    skip_tlv(data, &mut pos)?;

    // PDU (context-specific: 0xa2 = GetResponse)
    let pdu_tag = read_tag(data, &mut pos)?;
    if pdu_tag != 0xa2 {
        anyhow::bail!("SNMP: expected GetResponse PDU (0xa2), got 0x{:02x}", pdu_tag);
    }
    let _pdu_len = read_length(data, &mut pos)?;

    // request-id (INTEGER)
    skip_tlv(data, &mut pos)?;

    // error-status (INTEGER)
    let (_tag, error_status_bytes) = read_tlv_value(data, &mut pos)?;
    let error_status = if error_status_bytes.len() == 1 {
        error_status_bytes[0] as i32
    } else {
        0
    };
    if error_status != 0 {
        let error_name = match error_status {
            1 => "tooBig",
            2 => "noSuchName",
            3 => "badValue",
            4 => "readOnly",
            5 => "genErr",
            6 => "noAccess",
            7 => "wrongType",
            _ => "unknown",
        };
        anyhow::bail!("SNMP error-status: {} ({})", error_status, error_name);
    }

    // error-index (INTEGER)
    skip_tlv(data, &mut pos)?;

    // VarBindList (SEQUENCE)
    let tag = read_tag(data, &mut pos)?;
    if tag != 0x30 {
        anyhow::bail!("SNMP: expected VarBindList SEQUENCE, got 0x{:02x}", tag);
    }
    let varbind_list_len = read_length(data, &mut pos)?;
    let varbind_list_end = pos + varbind_list_len;

    let mut results = Vec::new();

    while pos < varbind_list_end && pos < data.len() {
        // VarBind (SEQUENCE)
        let tag = read_tag(data, &mut pos)?;
        if tag != 0x30 {
            break;
        }
        let _varbind_len = read_length(data, &mut pos)?;

        // OID
        let (oid_tag, oid_bytes) = read_tlv_value(data, &mut pos)?;
        if oid_tag != 0x06 {
            anyhow::bail!("SNMP: expected OID tag (0x06), got 0x{:02x}", oid_tag);
        }
        let oid = decode_oid(&oid_bytes)?;

        // Value
        let (value_tag, value_bytes) = read_tlv_value(data, &mut pos)?;

        // Skip endOfMibView, noSuchObject, noSuchInstance
        if value_tag == 0x82 || value_tag == 0x80 || value_tag == 0x81 {
            continue;
        }

        results.push((oid, value_bytes));
    }

    Ok(results)
}

// ==================== Public Parsing Functions ====================

fn parse_get_response(data: &[u8]) -> Result<Vec<u8>> {
    let varbinds = parse_snmp_response(data)?;
    varbinds.into_iter()
        .next()
        .map(|(_, value)| value)
        .ok_or_else(|| anyhow::anyhow!("SNMP GET: no varbind in response"))
}

fn parse_getnext_response(data: &[u8]) -> Result<(Vec<u32>, Vec<u8>)> {
    let varbinds = parse_snmp_response(data)?;
    varbinds.into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("SNMP GETNEXT: no varbind in response"))
}

fn parse_getbulk_response(data: &[u8]) -> Result<Vec<(Vec<u32>, Vec<u8>)>> {
    parse_snmp_response(data)
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
        1 => Some(data[0] as i8 as i64),
        2 => Some(i16::from_be_bytes([data[0], data[1]]) as i64),
        4 => Some(i32::from_be_bytes([data[0], data[1], data[2], data[3]]) as i64),
        8 => Some(i64::from_be_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]])),
        _ => None,
    }
}

fn parse_timeticks(data: &[u8]) -> Option<i64> {
    // TimeTicks is an unsigned 32-bit value
    match data.len() {
        4 => Some(u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as i64),
        _ => parse_integer(data),
    }
}

// ==================== PDU Builders ====================

fn build_get_pdu(version: &SnmpVersion, community: &str, oid: &[u32]) -> Vec<u8> {
    build_pdu(0xa0, version, community, oid, None) // GetRequest-PDU
}

fn build_getnext_pdu(version: &SnmpVersion, community: &str, oid: &[u32]) -> Vec<u8> {
    build_pdu(0xa1, version, community, oid, None) // GetNextRequest-PDU
}

fn build_getbulk_pdu(version: &SnmpVersion, community: &str, oid: &[u32], max_repetitions: u32) -> Vec<u8> {
    build_pdu(0xa5, version, community, oid, Some(max_repetitions)) // GetBulkRequest-PDU
}

fn build_pdu(pdu_type: u8, version: &SnmpVersion, community: &str, oid: &[u32], max_repetitions: Option<u32>) -> Vec<u8> {
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
    encode_length(&mut oid_bytes, oid_value.len());
    oid_bytes.extend_from_slice(&oid_value);

    // VarBind: OID + NULL value
    let mut varbind = Vec::new();
    varbind.push(0x30); // SEQUENCE
    let varbind_content_len = oid_bytes.len() + 2; // OID + NULL (05 00)
    encode_length(&mut varbind, varbind_content_len);
    varbind.extend_from_slice(&oid_bytes);
    varbind.extend_from_slice(&[0x05, 0x00]); // NULL

    // VarBindList
    let mut varbind_list = vec![0x30]; // SEQUENCE
    encode_length(&mut varbind_list, varbind.len());
    varbind_list.extend_from_slice(&varbind);

    // PDU
    let request_id: u32 = rand::random::<u16>() as u32;
    let mut pdu_content = Vec::new();
    // request-id
    pdu_content.extend_from_slice(&[0x02, 0x04]);
    pdu_content.extend_from_slice(&request_id.to_be_bytes());

    if let Some(max_rep) = max_repetitions {
        // GETBULK: non-repeaters = 0, max-repetitions = N
        pdu_content.extend_from_slice(&[0x02, 0x01, 0x00]); // non-repeaters = 0
        // max-repetitions
        pdu_content.push(0x02); // INTEGER
        let rep_bytes = max_rep.to_be_bytes();
        // Find first non-zero byte for minimal encoding
        let start = rep_bytes.iter().position(|&b| b != 0).unwrap_or(3);
        let significant = &rep_bytes[start..];
        pdu_content.push(significant.len() as u8);
        pdu_content.extend_from_slice(significant);
    } else {
        // error-status = 0
        pdu_content.extend_from_slice(&[0x02, 0x01, 0x00]);
        // error-index = 0
        pdu_content.extend_from_slice(&[0x02, 0x01, 0x00]);
    }
    pdu_content.extend_from_slice(&varbind_list);

    let mut pdu_bytes = vec![pdu_type];
    encode_length(&mut pdu_bytes, pdu_content.len());
    pdu_bytes.extend_from_slice(&pdu_content);

    // Community string
    let mut comm_bytes = vec![0x04]; // OCTET STRING
    encode_length(&mut comm_bytes, community.len());
    comm_bytes.extend_from_slice(community.as_bytes());

    // Message: version + community + PDU
    let mut message_content = Vec::new();
    // Version byte
    message_content.extend_from_slice(&[0x02, 0x01, version.version_byte()]);
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
