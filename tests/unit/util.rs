//! Unit tests for utility functions.

use netdisco::util;
use pretty_assertions::assert_eq;

// ==================== MAC Address Formatting ====================

#[test]
fn test_format_mac_ieee_bare() {
    assert_eq!(util::format_mac_ieee("001122334455"), "00:11:22:33:44:55");
}

#[test]
fn test_format_mac_ieee_colon_separated() {
    assert_eq!(util::format_mac_ieee("00:11:22:33:44:55"), "00:11:22:33:44:55");
}

#[test]
fn test_format_mac_ieee_dash_separated() {
    assert_eq!(util::format_mac_ieee("00-11-22-33-44-55"), "00:11:22:33:44:55");
}

#[test]
fn test_format_mac_ieee_dot_separated() {
    assert_eq!(util::format_mac_ieee("0011.2233.4455"), "00:11:22:33:44:55");
}

#[test]
fn test_format_mac_ieee_uppercase() {
    assert_eq!(util::format_mac_ieee("AABBCCDDEEFF"), "aa:bb:cc:dd:ee:ff");
}

#[test]
fn test_format_mac_ieee_mixed_case() {
    assert_eq!(util::format_mac_ieee("AaBbCcDdEeFf"), "aa:bb:cc:dd:ee:ff");
}

#[test]
fn test_format_mac_ieee_invalid_length_short() {
    // Should return as-is for non-12-hex-digit strings
    let result = util::format_mac_ieee("0011");
    assert_eq!(result, "0011");
}

#[test]
fn test_format_mac_ieee_invalid_chars() {
    let result = util::format_mac_ieee("not_a_mac_addr");
    assert_eq!(result, "not_a_mac_addr"); // Returned as-is
}

#[test]
fn test_format_mac_ieee_all_zeros() {
    assert_eq!(util::format_mac_ieee("000000000000"), "00:00:00:00:00:00");
}

#[test]
fn test_format_mac_ieee_all_ff() {
    assert_eq!(util::format_mac_ieee("FFFFFFFFFFFF"), "ff:ff:ff:ff:ff:ff");
}

// ==================== Uptime Formatting ====================

#[test]
fn test_format_uptime_zero() {
    assert_eq!(util::format_uptime(0), "00:00:00");
}

#[test]
fn test_format_uptime_one_second() {
    assert_eq!(util::format_uptime(100), "00:00:01");
}

#[test]
fn test_format_uptime_one_minute() {
    assert_eq!(util::format_uptime(6000), "00:01:00");
}

#[test]
fn test_format_uptime_one_hour() {
    assert_eq!(util::format_uptime(360000), "01:00:00");
}

#[test]
fn test_format_uptime_complex_time() {
    // 2 hours, 30 minutes, 45 seconds = 9045 seconds = 904500 ticks
    assert_eq!(util::format_uptime(904500), "02:30:45");
}

#[test]
fn test_format_uptime_one_day() {
    // 86400 seconds = 8640000 ticks
    assert_eq!(util::format_uptime(8640000), "1 day 00:00:00");
}

#[test]
fn test_format_uptime_multiple_days() {
    // 2 days + 5 hours = 2*86400 + 5*3600 = 190800 seconds = 19080000 ticks
    assert_eq!(util::format_uptime(19080000), "2 days 05:00:00");
}

#[test]
fn test_format_uptime_one_year() {
    // 366 days = 31622400 seconds = 3162240000 ticks
    let result = util::format_uptime(3162240000);
    assert!(result.contains("year"), "Expected 'year' in: {}", result);
}

#[test]
fn test_format_uptime_multiple_years() {
    // 730 days = 2 years
    let ticks: i64 = 730 * 86400 * 100;
    let result = util::format_uptime(ticks);
    assert!(result.contains("years"), "Expected 'years' in: {}", result);
}

// ==================== Permission / ACL Tests ====================

use netdisco::util::permission;
use ipnetwork::IpNetwork;

#[test]
fn test_acl_matches_direct_ip() {
    let ip: IpNetwork = "192.168.1.1/32".parse().unwrap();
    let acl = vec!["192.168.1.1".to_string()];
    assert!(permission::acl_matches(&ip, &acl));
}

#[test]
fn test_acl_matches_cidr() {
    let ip: IpNetwork = "192.168.1.50/32".parse().unwrap();
    let acl = vec!["192.168.1.0/24".to_string()];
    assert!(permission::acl_matches(&ip, &acl));
}

#[test]
fn test_acl_matches_cidr_no_match() {
    let ip: IpNetwork = "10.0.0.1/32".parse().unwrap();
    let acl = vec!["192.168.1.0/24".to_string()];
    assert!(!permission::acl_matches(&ip, &acl));
}

#[test]
fn test_acl_matches_any_wildcard() {
    let ip: IpNetwork = "10.0.0.1/32".parse().unwrap();
    let acl = vec!["group:__ANY__".to_string()];
    assert!(permission::acl_matches(&ip, &acl));
}

#[test]
fn test_acl_matches_ipv4_wildcard() {
    let ip: IpNetwork = "10.0.0.1/32".parse().unwrap();
    let acl = vec!["0.0.0.0/0".to_string()];
    assert!(permission::acl_matches(&ip, &acl));
}

#[test]
fn test_acl_matches_ipv6_wildcard() {
    let ip: IpNetwork = "10.0.0.1/32".parse().unwrap();
    let acl = vec!["::/0".to_string()];
    assert!(permission::acl_matches(&ip, &acl));
}

#[test]
fn test_acl_matches_empty_acl() {
    let ip: IpNetwork = "10.0.0.1/32".parse().unwrap();
    let acl: Vec<String> = vec![];
    assert!(!permission::acl_matches(&ip, &acl));
}

#[test]
fn test_acl_matches_multiple_entries() {
    let ip: IpNetwork = "172.16.0.5/32".parse().unwrap();
    let acl = vec![
        "10.0.0.0/8".to_string(),
        "172.16.0.0/24".to_string(),
        "192.168.0.0/16".to_string(),
    ];
    assert!(permission::acl_matches(&ip, &acl));
}

#[test]
fn test_acl_matches_only_empty_allows_all() {
    let ip: IpNetwork = "10.0.0.1/32".parse().unwrap();
    let only: Vec<String> = vec![];
    assert!(permission::acl_matches_only(&ip, &only));
}

#[test]
fn test_acl_matches_only_with_match() {
    let ip: IpNetwork = "10.0.0.1/32".parse().unwrap();
    let only = vec!["10.0.0.0/8".to_string()];
    assert!(permission::acl_matches_only(&ip, &only));
}

#[test]
fn test_acl_matches_only_no_match() {
    let ip: IpNetwork = "192.168.1.1/32".parse().unwrap();
    let only = vec!["10.0.0.0/8".to_string()];
    assert!(!permission::acl_matches_only(&ip, &only));
}

#[test]
fn test_acl_matches_no_empty_allows_all() {
    let ip: IpNetwork = "10.0.0.1/32".parse().unwrap();
    let no: Vec<String> = vec![];
    assert!(!permission::acl_matches_no(&ip, &no));
}

#[test]
fn test_acl_matches_no_with_match() {
    let ip: IpNetwork = "10.0.0.1/32".parse().unwrap();
    let no = vec!["10.0.0.0/8".to_string()];
    assert!(permission::acl_matches_no(&ip, &no));
}

#[test]
fn test_is_permitted_in_only_not_in_no() {
    let ip: IpNetwork = "10.0.0.1/32".parse().unwrap();
    let only = vec!["10.0.0.0/8".to_string()];
    let no: Vec<String> = vec![];
    assert!(permission::is_permitted(&ip, &only, &no));
}

#[test]
fn test_is_permitted_in_only_and_in_no() {
    let ip: IpNetwork = "10.0.0.1/32".parse().unwrap();
    let only = vec!["10.0.0.0/8".to_string()];
    let no = vec!["10.0.0.1".to_string()];
    assert!(!permission::is_permitted(&ip, &only, &no));
}

#[test]
fn test_is_permitted_not_in_only() {
    let ip: IpNetwork = "192.168.1.1/32".parse().unwrap();
    let only = vec!["10.0.0.0/8".to_string()];
    let no: Vec<String> = vec![];
    assert!(!permission::is_permitted(&ip, &only, &no));
}

#[test]
fn test_is_permitted_empty_both() {
    let ip: IpNetwork = "10.0.0.1/32".parse().unwrap();
    let only: Vec<String> = vec![];
    let no: Vec<String> = vec![];
    assert!(permission::is_permitted(&ip, &only, &no));
}

// ==================== Network Utilities ====================

use netdisco::util::net;
use std::net::IpAddr;

#[test]
fn test_is_private_ipv4_10() {
    let ip: IpAddr = "10.0.0.1".parse().unwrap();
    assert!(net::is_private(&ip));
}

#[test]
fn test_is_private_ipv4_172() {
    let ip: IpAddr = "172.16.0.1".parse().unwrap();
    assert!(net::is_private(&ip));
}

#[test]
fn test_is_private_ipv4_192() {
    let ip: IpAddr = "192.168.1.1".parse().unwrap();
    assert!(net::is_private(&ip));
}

#[test]
fn test_is_private_ipv4_loopback() {
    let ip: IpAddr = "127.0.0.1".parse().unwrap();
    assert!(net::is_private(&ip));
}

#[test]
fn test_is_private_ipv4_link_local() {
    let ip: IpAddr = "169.254.1.1".parse().unwrap();
    assert!(net::is_private(&ip));
}

#[test]
fn test_is_not_private_ipv4_public() {
    let ip: IpAddr = "8.8.8.8".parse().unwrap();
    assert!(!net::is_private(&ip));
}

#[test]
fn test_is_private_ipv6_loopback() {
    let ip: IpAddr = "::1".parse().unwrap();
    assert!(net::is_private(&ip));
}

#[test]
fn test_is_not_private_ipv6_public() {
    let ip: IpAddr = "2001:4860:4860::8888".parse().unwrap();
    assert!(!net::is_private(&ip));
}

// ==================== DNS Utility Tests ====================

use netdisco::util::dns;

#[tokio::test]
async fn test_hostname_from_ip_localhost() {
    let result = dns::hostname_from_ip("127.0.0.1").await;
    // May resolve to "localhost" or similar
    assert!(result.is_some() || result.is_none()); // Just ensure no panic
}

#[tokio::test]
async fn test_hostname_from_ip_invalid() {
    let result = dns::hostname_from_ip("not_an_ip").await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_ip_from_hostname_localhost() {
    let result = dns::ip_from_hostname("localhost").await;
    // Should resolve to 127.0.0.1 or ::1
    if let Some(ip) = &result {
        assert!(ip == "127.0.0.1" || ip == "::1", "Got: {}", ip);
    }
}

#[tokio::test]
async fn test_ip_from_hostname_invalid() {
    let result = dns::ip_from_hostname("host.invalid.tld.xyz.nonexistent").await;
    assert!(result.is_none());
}

// ==================== Ping Test ====================

#[tokio::test]
async fn test_ping_localhost() {
    use std::time::Duration;
    let result = net::ping_host("127.0.0.1", Duration::from_secs(2)).await;
    assert!(result, "Pinging localhost should succeed");
}
