//! netdisco-web: Web frontend server.

use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "netdisco-web", about = "Netdisco web frontend server")]
struct Cli {
    /// Configuration directory
    #[arg(short, long)]
    config: Option<String>,

    /// Port to listen on
    #[arg(short, long, default_value = "5000")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let cli = Cli::parse();

    let config = Arc::new(
        netdisco::config::load_config(cli.config.as_deref().map(std::path::Path::new))?
    );

    let db = netdisco::db::DbPool::new(&config.database).await?;
    db.ping().await?;

    tracing::info!("Netdisco {} web server starting on port {}", netdisco::VERSION, cli.port);

    netdisco::web::start_web_server(config, db.pool).await
}
