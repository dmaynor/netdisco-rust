//! netdisco-backend: Job control daemon.

use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "netdisco-backend", about = "Netdisco backend job control daemon")]
struct Cli {
    /// Configuration directory
    #[arg(short, long)]
    config: Option<String>,
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

    tracing::info!("Netdisco {} backend starting", netdisco::VERSION);

    netdisco::backend::start_backend(config, db.pool).await
}
