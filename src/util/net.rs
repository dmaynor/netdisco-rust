//! Network utilities.

use std::net::IpAddr;
use std::time::Duration;

/// Ping a host to check if it's reachable.
pub async fn ping_host(host: &str, timeout: Duration) -> bool {
    // Use system ping command
    let result = tokio::process::Command::new("ping")
        .args(["-c", "1", "-W", &timeout.as_secs().to_string(), host])
        .output()
        .await;

    match result {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// Check if an IP address is private/RFC1918.
pub fn is_private(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4.is_private() || v4.is_loopback() || v4.is_link_local(),
        IpAddr::V6(v6) => v6.is_loopback(),
    }
}
