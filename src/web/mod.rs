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
use actix_session::config::PersistentSession;
use actix_web::cookie::Key;
use actix_web::cookie::time::Duration as CookieDuration;
use actix_web::cookie::SameSite;
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

    // Use a persistent key from env, or generate and warn
    let secret_key = match std::env::var("NETDISCO_SESSION_KEY") {
        Ok(key_str) if key_str.len() >= 64 => {
            Key::from(key_str.as_bytes())
        }
        _ => {
            tracing::warn!("NETDISCO_SESSION_KEY not set or too short; generating ephemeral key (sessions lost on restart)");
            Key::generate()
        }
    };

    let app_state = web::Data::new(AppState {
        pool: pool.clone(),
        config: config.clone(),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    secret_key.clone(),
                )
                .cookie_http_only(true)
                .cookie_same_site(SameSite::Strict)
                .session_lifecycle(
                    PersistentSession::default()
                        .session_ttl(CookieDuration::hours(8))
                )
                .build()
            )
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
