//! SNMP client for device polling.
//!
//! Supports SNMP v1, v2c, and v3. Wraps the `snmp` crate with
//! Netdisco-specific functionality for device discovery, MAC table
//! collection, and ARP table collection.

pub mod client;
pub mod oids;

pub use client::*;
pub use oids::*;
