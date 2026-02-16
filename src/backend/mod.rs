//! Backend daemon - job processing and scheduled tasks.
//!
//! Ports the MCE-based Perl backend to a Tokio task-based system.

pub mod scheduler;
pub mod manager;

use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::signal;
use tracing::info;

use crate::config::NetdiscoConfig;
use crate::worker;

/// Start the backend daemon.
pub async fn start_backend(config: Arc<NetdiscoConfig>, pool: PgPool) -> Result<()> {
    info!("Starting Netdisco backend daemon");

    let num_workers = calculate_workers(&config.workers.tasks);
    info!("Starting {} worker tasks", num_workers);

    // Start the scheduler for periodic tasks
    let scheduler_config = config.clone();
    let scheduler_pool = pool.clone();
    let scheduler_handle = tokio::spawn(async move {
        scheduler::run_scheduler(scheduler_config, scheduler_pool).await
    });

    // Start worker tasks for job processing
    let mut worker_handles = Vec::new();
    for worker_id in 0..num_workers {
        let worker_config = config.clone();
        let worker_pool = pool.clone();
        let handle = tokio::spawn(async move {
            manager::run_worker(worker_id, worker_config, worker_pool).await
        });
        worker_handles.push(handle);
    }

    info!("Backend daemon running. Press Ctrl+C to stop.");

    // Wait for shutdown signal
    signal::ctrl_c().await?;
    info!("Shutdown signal received, stopping workers...");

    // Cancel all tasks
    scheduler_handle.abort();
    for handle in worker_handles {
        handle.abort();
    }

    info!("Backend daemon stopped");
    Ok(())
}

/// Calculate number of worker tasks from config string (e.g., "AUTO * 2").
fn calculate_workers(tasks_str: &str) -> usize {
    if tasks_str.starts_with("AUTO") {
        let cpus = num_cpus();
        if let Some(multiplier) = tasks_str.split('*').nth(1) {
            let mult: usize = multiplier.trim().parse().unwrap_or(2);
            cpus * mult
        } else {
            cpus * 2
        }
    } else {
        tasks_str.parse().unwrap_or(4)
    }
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2)
}
