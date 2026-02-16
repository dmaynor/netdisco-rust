//! Job scheduler - cron-like scheduling for periodic tasks.

use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{info, warn};

use crate::config::NetdiscoConfig;
use crate::db;

/// Run the scheduler loop, enqueuing periodic jobs.
pub async fn run_scheduler(config: Arc<NetdiscoConfig>, pool: PgPool) -> Result<()> {
    info!("Scheduler started");

    let mut tick = interval(Duration::from_secs(60)); // Check every minute

    loop {
        tick.tick().await;

        let now = chrono::Local::now();
        let minute = now.minute();
        let hour = now.hour();

        // discoverall: '5 7 * * *' (7:05 AM daily)
        if hour == 7 && minute == 5 {
            info!("Scheduling: discoverall");
            if let Err(e) = db::enqueue_job(&pool, "discoverall", None, None, Some("scheduler")).await {
                warn!("Failed to enqueue discoverall: {}", e);
            }
        }

        // macwalk: every 20 minutes
        if minute == 20 || minute == 40 || minute == 0 {
            info!("Scheduling: macwalk");
            if let Err(e) = db::enqueue_job(&pool, "macwalk", None, None, Some("scheduler")).await {
                warn!("Failed to enqueue macwalk: {}", e);
            }
        }

        // arpwalk: every 50 minutes
        if minute == 50 {
            info!("Scheduling: arpwalk");
            if let Err(e) = db::enqueue_job(&pool, "arpwalk", None, None, Some("scheduler")).await {
                warn!("Failed to enqueue arpwalk: {}", e);
            }
        }

        // nbtwalk: '0 8,13,21 * * *'
        if minute == 0 && (hour == 8 || hour == 13 || hour == 21) {
            info!("Scheduling: nbtwalk");
            if let Err(e) = db::enqueue_job(&pool, "nbtwalk", None, None, Some("scheduler")).await {
                warn!("Failed to enqueue nbtwalk: {}", e);
            }
        }

        // expire: '30 23 * * *'
        if hour == 23 && minute == 30 {
            info!("Scheduling: expire");
            if let Err(e) = db::enqueue_job(&pool, "expire", None, None, Some("scheduler")).await {
                warn!("Failed to enqueue expire: {}", e);
            }
        }
    }
}

use chrono::Timelike;
