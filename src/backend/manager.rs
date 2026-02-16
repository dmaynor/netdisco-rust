//! Worker manager - processes jobs from the queue.

use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error};

use crate::config::NetdiscoConfig;
use crate::db;
use crate::worker;

/// Run a single worker loop - dequeue and execute jobs.
pub async fn run_worker(worker_id: usize, config: Arc<NetdiscoConfig>, pool: PgPool) -> Result<()> {
    info!("Worker {} started", worker_id);

    let sleep_time = Duration::from_secs(config.workers.sleep_time);
    let timeout = Duration::from_secs(config.workers.timeout);

    loop {
        // Try to dequeue a job
        match db::dequeue_job(&pool).await {
            Ok(Some(job)) => {
                let job_id = job.job.unwrap_or(0);
                let action = job.action.as_deref().unwrap_or("unknown");
                info!("Worker {}: processing job {} ({})", worker_id, job_id, action);

                // Execute the job with a timeout
                let result = tokio::time::timeout(
                    timeout,
                    execute_job(&config, &pool, &job),
                ).await;

                match result {
                    Ok(Ok(log_msg)) => {
                        info!("Worker {}: job {} completed", worker_id, job_id);
                        if let Err(e) = db::complete_job(&pool, job_id, "done", &log_msg).await {
                            error!("Worker {}: failed to mark job {} as done: {}", worker_id, job_id, e);
                        }
                    }
                    Ok(Err(e)) => {
                        error!("Worker {}: job {} failed: {}", worker_id, job_id, e);
                        if let Err(db_err) = db::complete_job(&pool, job_id, "error", &e.to_string()).await {
                            error!("Worker {}: failed to mark job {} as error: {}", worker_id, job_id, db_err);
                        }
                    }
                    Err(_) => {
                        error!("Worker {}: job {} timed out", worker_id, job_id);
                        if let Err(e) = db::complete_job(&pool, job_id, "error", "Job timed out").await {
                            error!("Worker {}: failed to mark job {} as timed out: {}", worker_id, job_id, e);
                        }
                    }
                }
            }
            Ok(None) => {
                // No jobs available, sleep
                sleep(sleep_time).await;
            }
            Err(e) => {
                warn!("Worker {}: error dequeuing job: {}", worker_id, e);
                sleep(sleep_time).await;
            }
        }
    }
}

/// Execute a single job based on its action type.
async fn execute_job(
    config: &NetdiscoConfig,
    pool: &PgPool,
    job: &crate::models::Admin,
) -> Result<String> {
    let action = job.action.as_deref().unwrap_or("");
    let device_ip = job.device;

    match action {
        "discover" => {
            if let Some(ip) = device_ip {
                worker::discover::discover_device(config, pool, &ip).await
            } else {
                Err(anyhow::anyhow!("discover requires a device IP"))
            }
        }
        "discoverall" => {
            worker::discover::discover_all(config, pool).await
        }
        "macsuck" => {
            if let Some(ip) = device_ip {
                worker::macsuck::macsuck_device(config, pool, &ip).await
            } else {
                Err(anyhow::anyhow!("macsuck requires a device IP"))
            }
        }
        "macwalk" => {
            worker::macsuck::macwalk(config, pool).await
        }
        "arpnip" => {
            if let Some(ip) = device_ip {
                worker::arpnip::arpnip_device(config, pool, &ip).await
            } else {
                Err(anyhow::anyhow!("arpnip requires a device IP"))
            }
        }
        "arpwalk" => {
            worker::arpnip::arpwalk(config, pool).await
        }
        "nbtstat" => {
            worker::nbtstat::nbtstat_node(config, pool, job).await
        }
        "nbtwalk" => {
            worker::nbtstat::nbtwalk(config, pool).await
        }
        "expire" => {
            worker::expire::expire(config, pool).await
        }
        "delete" => {
            if let Some(ip) = device_ip {
                db::delete_device(pool, &ip).await?;
                Ok(format!("Deleted device {}", ip))
            } else {
                Err(anyhow::anyhow!("delete requires a device IP"))
            }
        }
        "portcontrol" | "portname" | "portvlan" | "power" => {
            worker::portcontrol::port_action(config, pool, job).await
        }
        "graph" | "show" | "stats" | "linter" => {
            Ok(format!("Action '{}' completed (placeholder)", action))
        }
        _ => {
            warn!("Unknown action: {}", action);
            Err(anyhow::anyhow!("Unknown action: {}", action))
        }
    }
}
