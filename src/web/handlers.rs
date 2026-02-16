//! Web request handlers.

use actix_web::{web, HttpResponse, HttpRequest};
use actix_session::Session;
use serde::Deserialize;

use super::AppState;
use crate::db;

// ==================== Page Handlers ====================

pub async fn index(state: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Found()
        .insert_header(("Location", state.config.web_home.as_str()))
        .finish()
}

pub async fn inventory(state: web::Data<AppState>) -> HttpResponse {
    let devices = db::list_devices(&state.pool, Some(100)).await.unwrap_or_default();
    let device_count = db::device_count(&state.pool).await.unwrap_or(0);
    let node_count = db::node_count(&state.pool, true).await.unwrap_or(0);
    let port_count = db::port_count(&state.pool).await.unwrap_or(0);

    HttpResponse::Ok().json(serde_json::json!({
        "devices": devices,
        "statistics": {
            "device_count": device_count,
            "node_count": node_count,
            "port_count": port_count,
        }
    }))
}

#[derive(Deserialize)]
pub struct DeviceQuery {
    pub q: Option<String>,
}

pub async fn device_search(
    state: web::Data<AppState>,
    query: web::Query<DeviceQuery>,
) -> HttpResponse {
    if let Some(q) = &query.q {
        let devices = db::search_devices(&state.pool, q).await.unwrap_or_default();
        HttpResponse::Ok().json(devices)
    } else {
        let devices = db::list_devices(&state.pool, Some(100)).await.unwrap_or_default();
        HttpResponse::Ok().json(devices)
    }
}

pub async fn device_detail(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let ip_str = path.into_inner();
    let ip: ipnetwork::IpNetwork = match ip_str.parse() {
        Ok(ip) => ip,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({"error": "Invalid IP address"})),
    };

    match db::find_device(&state.pool, &ip).await {
        Ok(Some(device)) => HttpResponse::Ok().json(device),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({"error": "Device not found"})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn device_ports(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let ip: ipnetwork::IpNetwork = match path.into_inner().parse() {
        Ok(ip) => ip,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({"error": "Invalid IP"})),
    };
    let ports = db::get_device_ports(&state.pool, &ip).await.unwrap_or_default();
    HttpResponse::Ok().json(ports)
}

pub async fn device_modules(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let ip: ipnetwork::IpNetwork = match path.into_inner().parse() {
        Ok(ip) => ip,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({"error": "Invalid IP"})),
    };
    let modules = db::get_device_modules(&state.pool, &ip).await.unwrap_or_default();
    HttpResponse::Ok().json(modules)
}

pub async fn device_neighbors(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let ip: ipnetwork::IpNetwork = match path.into_inner().parse() {
        Ok(ip) => ip,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({"error": "Invalid IP"})),
    };
    // Get ports that have remote_ip set (neighbors)
    let ports = db::get_device_ports(&state.pool, &ip).await.unwrap_or_default();
    let neighbors: Vec<_> = ports.into_iter().filter(|p| p.remote_ip.is_some()).collect();
    HttpResponse::Ok().json(neighbors)
}

pub async fn device_addresses(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let ip: ipnetwork::IpNetwork = match path.into_inner().parse() {
        Ok(ip) => ip,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({"error": "Invalid IP"})),
    };
    let ips = db::get_device_ips(&state.pool, &ip).await.unwrap_or_default();
    HttpResponse::Ok().json(ips)
}

pub async fn device_vlans(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let ip: ipnetwork::IpNetwork = match path.into_inner().parse() {
        Ok(ip) => ip,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({"error": "Invalid IP"})),
    };
    let vlans = db::get_device_vlans(&state.pool, &ip).await.unwrap_or_default();
    HttpResponse::Ok().json(vlans)
}

// ==================== Search Handlers ====================

#[derive(Deserialize)]
pub struct NodeSearch {
    pub q: Option<String>,
}

pub async fn search_node(
    state: web::Data<AppState>,
    query: web::Query<NodeSearch>,
) -> HttpResponse {
    if let Some(q) = &query.q {
        // Try MAC search first, then IP search
        if q.contains(':') || q.contains('-') || q.contains('.') {
            let nodes = db::find_node_by_mac(&state.pool, q).await.unwrap_or_default();
            return HttpResponse::Ok().json(nodes);
        }
        // Try as IP
        if let Ok(ip) = q.parse::<ipnetwork::IpNetwork>() {
            let nodes = db::find_node_by_ip(&state.pool, &ip).await.unwrap_or_default();
            return HttpResponse::Ok().json(nodes);
        }
    }
    HttpResponse::Ok().json(serde_json::json!([]))
}

pub async fn search_device(
    state: web::Data<AppState>,
    query: web::Query<DeviceQuery>,
) -> HttpResponse {
    if let Some(q) = &query.q {
        let devices = db::search_devices(&state.pool, q).await.unwrap_or_default();
        HttpResponse::Ok().json(devices)
    } else {
        HttpResponse::Ok().json(serde_json::json!([]))
    }
}

pub async fn search_vlan() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"status": "not_implemented"}))
}

pub async fn search_port() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"status": "not_implemented"}))
}

// ==================== Report & Admin Handlers ====================

pub async fn report(path: web::Path<String>) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"report": path.into_inner(), "status": "not_implemented"}))
}

pub async fn admin_job_queue(state: web::Data<AppState>) -> HttpResponse {
    let jobs = db::list_jobs(&state.pool, 50).await.unwrap_or_default();
    HttpResponse::Ok().json(jobs)
}

pub async fn admin_users() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"status": "not_implemented"}))
}

// ==================== Auth Handlers ====================

pub async fn login_page() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"page": "login"}))
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

pub async fn login_submit(
    state: web::Data<AppState>,
    session: Session,
    form: web::Form<LoginForm>,
) -> HttpResponse {
    match db::find_user(&state.pool, &form.username).await {
        Ok(Some(user)) => {
            // Verify password
            if let Some(ref stored_hash) = user.password {
                match bcrypt::verify(&form.password, stored_hash) {
                    Ok(true) => {
                        session.insert("username", &user.username).ok();
                        session.insert("admin", user.is_admin()).ok();
                        HttpResponse::Found()
                            .insert_header(("Location", "/"))
                            .finish()
                    }
                    _ => HttpResponse::Unauthorized().json(serde_json::json!({"error": "Invalid credentials"})),
                }
            } else {
                HttpResponse::Unauthorized().json(serde_json::json!({"error": "No password set"}))
            }
        }
        _ => HttpResponse::Unauthorized().json(serde_json::json!({"error": "Invalid credentials"})),
    }
}

pub async fn logout(session: Session) -> HttpResponse {
    session.purge();
    HttpResponse::Found()
        .insert_header(("Location", "/login"))
        .finish()
}

pub async fn change_password() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"status": "not_implemented"}))
}
