//! DNS resolution utilities.

use std::net::IpAddr;

/// Resolve an IP address to a hostname.
pub async fn hostname_from_ip(ip: &str) -> Option<String> {
    let addr: IpAddr = ip.parse().ok()?;
    tokio::task::spawn_blocking(move || {
        dns_lookup::lookup_addr(&addr).ok()
    }).await.ok()?
}

/// Resolve a hostname to an IP address.
pub async fn ip_from_hostname(hostname: &str) -> Option<String> {
    let hostname = hostname.to_string();
    tokio::task::spawn_blocking(move || {
        use std::net::ToSocketAddrs;
        format!("{}:0", hostname)
            .to_socket_addrs()
            .ok()?
            .next()
            .map(|addr| addr.ip().to_string())
    }).await.ok()?
}
