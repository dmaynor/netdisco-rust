//! Utility functions shared across the application.

pub mod dns;
pub mod permission;
pub mod net;

/// Format a MAC address into standard IEEE format (00:11:22:33:44:55).
pub fn format_mac_ieee(mac: &str) -> String {
    let clean: String = mac.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if clean.len() != 12 {
        return mac.to_string();
    }
    clean.as_bytes()
        .chunks(2)
        .map(|chunk| std::str::from_utf8(chunk).unwrap_or("00"))
        .collect::<Vec<&str>>()
        .join(":")
        .to_lowercase()
}

/// Format uptime ticks (hundredths of a second) into human-readable string.
pub fn format_uptime(ticks: i64) -> String {
    let seconds = ticks / 100;
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if days > 365 {
        let years = days / 365;
        let remaining_days = days % 365;
        format!("{} year{} {} day{} {:02}:{:02}:{:02}",
            years, if years != 1 { "s" } else { "" },
            remaining_days, if remaining_days != 1 { "s" } else { "" },
            hours, minutes, secs)
    } else if days > 0 {
        format!("{} day{} {:02}:{:02}:{:02}",
            days, if days != 1 { "s" } else { "" },
            hours, minutes, secs)
    } else {
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    }
}
