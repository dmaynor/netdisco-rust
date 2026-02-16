//! Permission/ACL checking utilities.

use ipnetwork::IpNetwork;

/// Check if an IP matches an ACL entry (IP, CIDR, or group reference).
pub fn acl_matches(ip: &IpNetwork, acl: &[String]) -> bool {
    for entry in acl {
        if entry == "group:__ANY__" || entry == "0.0.0.0/0" || entry == "::/0" {
            return true;
        }
        if let Ok(network) = entry.parse::<IpNetwork>() {
            if network.contains(ip.ip()) {
                return true;
            }
        }
        // Direct IP match
        if entry == &ip.ip().to_string() {
            return true;
        }
    }
    false
}

/// Check if an IP is in the "only" list (empty means all allowed).
pub fn acl_matches_only(ip: &IpNetwork, only: &[String]) -> bool {
    if only.is_empty() {
        return true;
    }
    acl_matches(ip, only)
}

/// Check if an IP is in the "no" list.
pub fn acl_matches_no(ip: &IpNetwork, no: &[String]) -> bool {
    if no.is_empty() {
        return false;
    }
    acl_matches(ip, no)
}

/// Check if a device should be included based on only/no lists.
pub fn is_permitted(ip: &IpNetwork, only: &[String], no: &[String]) -> bool {
    if acl_matches_no(ip, no) {
        return false;
    }
    acl_matches_only(ip, only)
}
