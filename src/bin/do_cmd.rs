//! netdisco-do: Command-line tool for ad-hoc operations.

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "netdisco-do", about = "Netdisco command-line interface")]
struct Cli {
    /// Configuration directory
    #[arg(short, long)]
    config: Option<String>,

    /// Enable debug output
    #[arg(short = 'D', long)]
    debug: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Discover a device by IP or hostname
    Discover {
        /// Device IP or hostname
        #[arg(short, long)]
        device: String,
    },
    /// Collect MAC address table from a device
    Macsuck {
        #[arg(short, long)]
        device: String,
    },
    /// Collect ARP table from a device
    Arpnip {
        #[arg(short, long)]
        device: String,
    },
    /// Delete a device from the database
    Delete {
        #[arg(short, long)]
        device: String,
    },
    /// Show device information
    Show {
        #[arg(short, long)]
        device: String,
    },
    /// Run scheduled tasks
    DiscoverAll,
    Macwalk,
    Arpwalk,
    Nbtwalk,
    Expire,
    /// Dump current configuration
    DumpConfig,
    /// Show database statistics
    Stats,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let filter = if cli.debug { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(filter.parse()?))
        .init();

    let config = Arc::new(
        netdisco::config::load_config(cli.config.as_deref().map(std::path::Path::new))?
    );

    let db = netdisco::db::DbPool::new(&config.database).await?;
    db.ping().await?;

    match cli.command {
        Commands::Discover { device } => {
            let ip = resolve_device(&device).await?;
            let result = netdisco::worker::discover::discover_device(&config, &db.pool, &ip).await?;
            println!("{}", result);
        }
        Commands::Macsuck { device } => {
            let ip = resolve_device(&device).await?;
            let result = netdisco::worker::macsuck::macsuck_device(&config, &db.pool, &ip).await?;
            println!("{}", result);
        }
        Commands::Arpnip { device } => {
            let ip = resolve_device(&device).await?;
            let result = netdisco::worker::arpnip::arpnip_device(&config, &db.pool, &ip).await?;
            println!("{}", result);
        }
        Commands::Delete { device } => {
            let ip = resolve_device(&device).await?;
            netdisco::db::delete_device(&db.pool, &ip).await?;
            println!("Deleted device {}", ip);
        }
        Commands::Show { device } => {
            let ip = resolve_device(&device).await?;
            match netdisco::db::find_device(&db.pool, &ip).await? {
                Some(dev) => {
                    println!("Device: {}", dev.display_name());
                    println!("  IP: {}", dev.ip);
                    println!("  DNS: {}", dev.dns.as_deref().unwrap_or("(none)"));
                    println!("  Description: {}", dev.description.as_deref().unwrap_or("(none)"));
                    println!("  Location: {}", dev.location.as_deref().unwrap_or("(none)"));
                    println!("  Vendor: {}", dev.vendor.as_deref().unwrap_or("(none)"));
                    println!("  Model: {}", dev.model.as_deref().unwrap_or("(none)"));
                    println!("  OS: {} {}", dev.os.as_deref().unwrap_or(""), dev.os_ver.as_deref().unwrap_or(""));
                    println!("  Serial: {}", dev.serial.as_deref().unwrap_or("(none)"));
                    println!("  Layers: {}", dev.layers.as_deref().unwrap_or("(none)"));
                    if let Some(uptime) = dev.uptime {
                        println!("  Uptime: {}", netdisco::util::format_uptime(uptime));
                    }
                }
                None => println!("Device {} not found", ip),
            }
        }
        Commands::DiscoverAll => {
            let result = netdisco::worker::discover::discover_all(&config, &db.pool).await?;
            println!("{}", result);
        }
        Commands::Macwalk => {
            let result = netdisco::worker::macsuck::macwalk(&config, &db.pool).await?;
            println!("{}", result);
        }
        Commands::Arpwalk => {
            let result = netdisco::worker::arpnip::arpwalk(&config, &db.pool).await?;
            println!("{}", result);
        }
        Commands::Nbtwalk => {
            let result = netdisco::worker::nbtstat::nbtwalk(&config, &db.pool).await?;
            println!("{}", result);
        }
        Commands::Expire => {
            let result = netdisco::worker::expire::expire(&config, &db.pool).await?;
            println!("{}", result);
        }
        Commands::DumpConfig => {
            println!("{}", serde_yaml::to_string(&*config)?);
        }
        Commands::Stats => {
            let device_count = netdisco::db::device_count(&db.pool).await?;
            let node_count = netdisco::db::node_count(&db.pool, true).await?;
            let port_count = netdisco::db::port_count(&db.pool).await?;
            println!("Database Statistics:");
            println!("  Devices: {}", device_count);
            println!("  Active Nodes: {}", node_count);
            println!("  Ports: {}", port_count);
        }
    }

    Ok(())
}

async fn resolve_device(device: &str) -> Result<ipnetwork::IpNetwork> {
    // Try parsing as IP first
    if let Ok(ip) = device.parse::<std::net::IpAddr>() {
        return Ok(ipnetwork::IpNetwork::from(ip));
    }
    // Try DNS resolution
    if let Some(ip_str) = netdisco::util::dns::ip_from_hostname(device).await {
        if let Ok(ip) = ip_str.parse::<std::net::IpAddr>() {
            return Ok(ipnetwork::IpNetwork::from(ip));
        }
    }
    Err(anyhow::anyhow!("Cannot resolve device: {}", device))
}
