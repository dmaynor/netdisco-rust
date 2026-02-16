//! Web server module - Actix-web based HTTP server and REST API.
//!
//! Ports the Perl Dancer-based web frontend to Actix-web.

pub mod routes;
pub mod auth;
pub mod api;
pub mod handlers;

use actix_web::{web, App, HttpServer, middleware};
use actix_files as fs;
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::cookie::Key;
use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::info;

use crate::config::NetdiscoConfig;

/// Application state shared across all web handlers.
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<NetdiscoConfig>,
}

/// Start the web server.
pub async fn start_web_server(config: Arc<NetdiscoConfig>, pool: PgPool) -> Result<()> {
    let bind_addr = format!("0.0.0.0:{}", crate::DEFAULT_WEB_PORT);
    info!("Starting Netdisco web server on {}", bind_addr);

    let secret_key = Key::generate();
    let app_state = web::Data::new(AppState {
        pool: pool.clone(),
        config: config.clone(),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(SessionMiddleware::builder(
                CookieSessionStore::default(),
                secret_key.clone(),
            ).build())
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            // Static files
            .service(fs::Files::new("/static", "./share/public").show_files_listing())
            // API routes
            .configure(api::configure)
            // Web routes
            .configure(routes::configure)
    })
    .bind(&bind_addr)?
    .run()
    .await?;

    Ok(())
}
