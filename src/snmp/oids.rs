//! Standard SNMP OID definitions used by Netdisco.

// System MIB (RFC 1213)
pub const SYS_DESCR: [u32; 8] = [1, 3, 6, 1, 2, 1, 1, 1];
pub const SYS_OBJECT_ID: [u32; 8] = [1, 3, 6, 1, 2, 1, 1, 2];
pub const SYS_UPTIME: [u32; 8] = [1, 3, 6, 1, 2, 1, 1, 3];
pub const SYS_CONTACT: [u32; 8] = [1, 3, 6, 1, 2, 1, 1, 4];
pub const SYS_NAME: [u32; 8] = [1, 3, 6, 1, 2, 1, 1, 5];
pub const SYS_LOCATION: [u32; 8] = [1, 3, 6, 1, 2, 1, 1, 6];
pub const SYS_SERVICES: [u32; 8] = [1, 3, 6, 1, 2, 1, 1, 7];

// Interfaces MIB (IF-MIB)
pub const IF_NUMBER: [u32; 8] = [1, 3, 6, 1, 2, 1, 2, 1];
pub const IF_INDEX: [u32; 10] = [1, 3, 6, 1, 2, 1, 2, 2, 1, 1];
pub const IF_DESCR: [u32; 10] = [1, 3, 6, 1, 2, 1, 2, 2, 1, 2];
pub const IF_TYPE: [u32; 10] = [1, 3, 6, 1, 2, 1, 2, 2, 1, 3];
pub const IF_MTU: [u32; 10] = [1, 3, 6, 1, 2, 1, 2, 2, 1, 4];
pub const IF_SPEED: [u32; 10] = [1, 3, 6, 1, 2, 1, 2, 2, 1, 5];
pub const IF_PHYS_ADDRESS: [u32; 10] = [1, 3, 6, 1, 2, 1, 2, 2, 1, 6];
pub const IF_ADMIN_STATUS: [u32; 10] = [1, 3, 6, 1, 2, 1, 2, 2, 1, 7];
pub const IF_OPER_STATUS: [u32; 10] = [1, 3, 6, 1, 2, 1, 2, 2, 1, 8];
pub const IF_LAST_CHANGE: [u32; 10] = [1, 3, 6, 1, 2, 1, 2, 2, 1, 9];

// ifXTable (IF-MIB)
pub const IF_NAME: [u32; 10] = [1, 3, 6, 1, 2, 1, 31, 1, 1, 1];
pub const IF_HIGH_SPEED: [u32; 11] = [1, 3, 6, 1, 2, 1, 31, 1, 1, 1, 15];
pub const IF_ALIAS: [u32; 11] = [1, 3, 6, 1, 2, 1, 31, 1, 1, 1, 18];

// Bridge MIB (BRIDGE-MIB) - MAC address table
pub const DOT1D_TP_FDB_ADDRESS: [u32; 10] = [1, 3, 6, 1, 2, 1, 17, 4, 3, 1];
pub const DOT1D_TP_FDB_PORT: [u32; 11] = [1, 3, 6, 1, 2, 1, 17, 4, 3, 1, 2];
pub const DOT1D_TP_FDB_STATUS: [u32; 11] = [1, 3, 6, 1, 2, 1, 17, 4, 3, 1, 3];

// Q-BRIDGE-MIB - VLAN-aware MAC table
pub const DOT1Q_TP_FDB_PORT: [u32; 11] = [1, 3, 6, 1, 2, 1, 17, 7, 1, 2, 2];

// IP MIB - ARP table
pub const IP_NET_TO_MEDIA_PHYS: [u32; 10] = [1, 3, 6, 1, 2, 1, 4, 22, 1, 2];
pub const IP_NET_TO_MEDIA_TYPE: [u32; 10] = [1, 3, 6, 1, 2, 1, 4, 22, 1, 4];

// Entity MIB (ENTITY-MIB) - modules/inventory
pub const ENT_PHYSICAL_DESCR: [u32; 10] = [1, 3, 6, 1, 2, 1, 47, 1, 1, 1];
pub const ENT_PHYSICAL_CLASS: [u32; 11] = [1, 3, 6, 1, 2, 1, 47, 1, 1, 1, 5];
pub const ENT_PHYSICAL_NAME: [u32; 11] = [1, 3, 6, 1, 2, 1, 47, 1, 1, 1, 7];
pub const ENT_PHYSICAL_HW_REV: [u32; 11] = [1, 3, 6, 1, 2, 1, 47, 1, 1, 1, 8];
pub const ENT_PHYSICAL_FW_REV: [u32; 11] = [1, 3, 6, 1, 2, 1, 47, 1, 1, 1, 9];
pub const ENT_PHYSICAL_SW_REV: [u32; 11] = [1, 3, 6, 1, 2, 1, 47, 1, 1, 1, 10];
pub const ENT_PHYSICAL_SERIAL: [u32; 11] = [1, 3, 6, 1, 2, 1, 47, 1, 1, 1, 11];
pub const ENT_PHYSICAL_MODEL: [u32; 11] = [1, 3, 6, 1, 2, 1, 47, 1, 1, 1, 13];

// LLDP MIB (LLDP-MIB) - neighbor discovery
pub const LLDP_REM_SYS_NAME: [u32; 10] = [1, 0, 8802, 1, 1, 2, 1, 4, 1, 9];
pub const LLDP_REM_SYS_DESC: [u32; 10] = [1, 0, 8802, 1, 1, 2, 1, 4, 1, 10];
pub const LLDP_REM_PORT_ID: [u32; 10] = [1, 0, 8802, 1, 1, 2, 1, 4, 1, 7];
pub const LLDP_REM_MAN_ADDR: [u32; 10] = [1, 0, 8802, 1, 1, 2, 1, 4, 2, 1];

// CDP MIB (CISCO-CDP-MIB) - Cisco neighbor discovery
pub const CDP_CACHE_DEVICE_ID: [u32; 11] = [1, 3, 6, 1, 4, 1, 9, 9, 23, 1, 2];
pub const CDP_CACHE_DEVICE_PORT: [u32; 12] = [1, 3, 6, 1, 4, 1, 9, 9, 23, 1, 2, 1];
pub const CDP_CACHE_ADDRESS: [u32; 12] = [1, 3, 6, 1, 4, 1, 9, 9, 23, 1, 2, 4];
pub const CDP_CACHE_PLATFORM: [u32; 12] = [1, 3, 6, 1, 4, 1, 9, 9, 23, 1, 2, 8];

// VLAN MIB
pub const VTP_VLAN_STATE: [u32; 12] = [1, 3, 6, 1, 4, 1, 9, 9, 46, 1, 3, 1];
pub const VTP_VLAN_NAME: [u32; 13] = [1, 3, 6, 1, 4, 1, 9, 9, 46, 1, 3, 1, 4];
pub const DOT1Q_VLAN_STATIC_NAME: [u32; 11] = [1, 3, 6, 1, 2, 1, 17, 7, 1, 4, 3];

// PoE MIB (POWER-ETHERNET-MIB)
pub const PSE_PORT_ADMIN: [u32; 11] = [1, 3, 6, 1, 2, 1, 105, 1, 1, 1, 3];
pub const PSE_PORT_STATUS: [u32; 11] = [1, 3, 6, 1, 2, 1, 105, 1, 1, 1, 6];
pub const PSE_PORT_POWER: [u32; 11] = [1, 3, 6, 1, 2, 1, 105, 1, 1, 1, 7];
