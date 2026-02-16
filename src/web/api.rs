//! REST API endpoints (v1).
//!
//! Provides a JSON API compatible with the original Netdisco Swagger API.

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/object/device", web::get().to(api_list_devices))
            .route("/object/device/{ip}", web::get().to(api_get_device))
            .route("/object/device/{ip}/ports", web::get().to(api_device_ports))
            .route("/search/node", web::get().to(api_search_node))
            .route("/search/device", web::get().to(api_search_device))
            .route("/queue", web::get().to(api_list_jobs))
            .route("/queue", web::post().to(api_enqueue_job))
    );
}

use actix_web::HttpResponse;
use actix_session::Session;
use serde::Deserialize;
use tracing::error;
use crate::db;
use super::auth;

async fn api_list_devices(state: web::Data<super::AppState>, session: Session) -> HttpResponse {
    if let Some(resp) = auth::require_auth(&session, &state.config) {
        return resp;
    }
    let devices = db::list_devices(&state.pool, Some(1000)).await.unwrap_or_default();
    HttpResponse::Ok().json(devices)
}

async fn api_get_device(
    state: web::Data<super::AppState>,
    session: Session,
    path: web::Path<String>,
) -> HttpResponse {
    if let Some(resp) = auth::require_auth(&session, &state.config) {
        return resp;
    }
    let ip: ipnetwork::IpNetwork = match path.into_inner().parse() {
        Ok(ip) => ip,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({"error": "Invalid IP"})),
    };
    match db::find_device(&state.pool, &ip).await {
        Ok(Some(device)) => HttpResponse::Ok().json(device),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({"error": "Not found"})),
        Err(e) => {
            error!("Database error in api_get_device: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({"error": "Internal server error"}))
        }
    }
}

async fn api_device_ports(
    state: web::Data<super::AppState>,
    session: Session,
    path: web::Path<String>,
) -> HttpResponse {
    if let Some(resp) = auth::require_auth(&session, &state.config) {
        return resp;
    }
    let ip: ipnetwork::IpNetwork = match path.into_inner().parse() {
        Ok(ip) => ip,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({"error": "Invalid IP"})),
    };
    let ports = db::get_device_ports(&state.pool, &ip).await.unwrap_or_default();
    HttpResponse::Ok().json(ports)
}

async fn api_search_node(
    state: web::Data<super::AppState>,
    session: Session,
    query: web::Query<super::handlers::NodeSearch>,
) -> HttpResponse {
    super::handlers::search_node(state, session, query).await
}

async fn api_search_device(
    state: web::Data<super::AppState>,
    session: Session,
    query: web::Query<super::handlers::DeviceQuery>,
) -> HttpResponse {
    super::handlers::search_device(state, session, query).await
}

async fn api_list_jobs(state: web::Data<super::AppState>, session: Session) -> HttpResponse {
    if let Some(resp) = auth::require_auth(&session, &state.config) {
        return resp;
    }
    let jobs = db::list_jobs(&state.pool, 100).await.unwrap_or_default();
    HttpResponse::Ok().json(jobs)
}

#[derive(Deserialize)]
struct JobRequest {
    action: String,
    device: Option<String>,
    port: Option<String>,
}

async fn api_enqueue_job(
    state: web::Data<super::AppState>,
    session: Session,
    body: web::Json<JobRequest>,
) -> HttpResponse {
    // Require admin for job creation
    if let Some(resp) = auth::require_admin(&session, &state.config) {
        return resp;
    }

    // Validate action against allowed list
    if !auth::is_valid_job_action(&body.action) {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "Invalid action type"}));
    }

    let device_ip = body.device.as_ref().and_then(|d| d.parse().ok());
    match db::enqueue_job(
        &state.pool,
        &body.action,
        device_ip.as_ref(),
        body.port.as_deref(),
        None,
    ).await {
        Ok(job_id) => HttpResponse::Ok().json(serde_json::json!({"job": job_id})),
        Err(e) => {
            error!("Failed to enqueue job: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to enqueue job"}))
        }
    }
}
