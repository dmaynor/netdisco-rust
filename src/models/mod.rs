//! Database models for all Netdisco entities.
//!
//! Each model maps to a PostgreSQL table and uses SQLx for query binding
//! and Serde for JSON serialization.

pub mod device;
pub mod device_browser;
pub mod device_ip;
pub mod device_module;
pub mod device_port;
pub mod device_port_log;
pub mod device_port_power;
pub mod device_port_ssid;
pub mod device_port_vlan;
pub mod device_port_wireless;
pub mod device_power;
pub mod device_skip;
pub mod device_vlan;
pub mod node;
pub mod node_ip;
pub mod node_monitor;
pub mod node_nbt;
pub mod node_wireless;
pub mod admin;
pub mod enterprise;
pub mod log;
pub mod manufacturer;
pub mod oui;
pub mod process;
pub mod session;
pub mod snmp_object;
pub mod statistics;
pub mod subnet;
pub mod topology;
pub mod user;
pub mod user_log;

// Re-export commonly used models
pub use device::Device;
pub use device_ip::DeviceIp;
pub use device_module::DeviceModule;
pub use device_port::DevicePort;
pub use device_port_log::DevicePortLog;
pub use device_port_power::DevicePortPower;
pub use device_port_ssid::DevicePortSsid;
pub use device_port_vlan::DevicePortVlan;
pub use device_port_wireless::DevicePortWireless;
pub use device_power::DevicePower;
pub use device_skip::DeviceSkip;
pub use device_vlan::DeviceVlan;
pub use node::Node;
pub use node_ip::NodeIp;
pub use node_monitor::NodeMonitor;
pub use node_nbt::NodeNbt;
pub use node_wireless::NodeWireless;
pub use admin::Admin;
pub use oui::Oui;
pub use user::User;
