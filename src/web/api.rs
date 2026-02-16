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
use serde::Deserialize;
use crate::db;

async fn api_list_devices(state: web::Data<super::AppState>) -> HttpResponse {
    let devices = db::list_devices(&state.pool, Some(1000)).await.unwrap_or_default();
    HttpResponse::Ok().json(devices)
}

async fn api_get_device(
    state: web::Data<super::AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let ip: ipnetwork::IpNetwork = match path.into_inner().parse() {
        Ok(ip) => ip,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({"error": "Invalid IP"})),
    };
    match db::find_device(&state.pool, &ip).await {
        Ok(Some(device)) => HttpResponse::Ok().json(device),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({"error": "Not found"})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

async fn api_device_ports(
    state: web::Data<super::AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let ip: ipnetwork::IpNetwork = match path.into_inner().parse() {
        Ok(ip) => ip,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({"error": "Invalid IP"})),
    };
    let ports = db::get_device_ports(&state.pool, &ip).await.unwrap_or_default();
    HttpResponse::Ok().json(ports)
}

async fn api_search_node(
    state: web::Data<super::AppState>,
    query: web::Query<super::handlers::NodeSearch>,
) -> HttpResponse {
    super::handlers::search_node(state, query).await
}

async fn api_search_device(
    state: web::Data<super::AppState>,
    query: web::Query<super::handlers::DeviceQuery>,
) -> HttpResponse {
    super::handlers::search_device(state, query).await
}

async fn api_list_jobs(state: web::Data<super::AppState>) -> HttpResponse {
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
    body: web::Json<JobRequest>,
) -> HttpResponse {
    let device_ip = body.device.as_ref().and_then(|d| d.parse().ok());
    match db::enqueue_job(
        &state.pool,
        &body.action,
        device_ip.as_ref(),
        body.port.as_deref(),
        None,
    ).await {
        Ok(job_id) => HttpResponse::Ok().json(serde_json::json!({"job": job_id})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}
