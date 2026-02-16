//! netdisco-deploy: Database deployment and migration tool.

use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "netdisco-deploy", about = "Netdisco database deployment tool")]
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

    println!("Netdisco {} Database Deployment", netdisco::VERSION);
    println!("================================");

    let db = netdisco::db::DbPool::new(&config.database).await?;

    println!("Running database migrations...");
    netdisco::db::run_migrations(&db.pool).await?;

    let version = netdisco::db::schema_version(&db.pool).await?;
    println!("Database schema version: {}", version);

    // Create default admin user if none exists
    let admin_exists = netdisco::db::find_user(&db.pool, "admin").await?;
    if admin_exists.is_none() {
        println!("Creating default admin user (username: admin, password: admin)");
        let hash = bcrypt::hash("admin", bcrypt::DEFAULT_COST)?;
        netdisco::db::create_user(&db.pool, "admin", &hash, true).await?;
    }

    println!("Deployment complete!");
    Ok(())
}
