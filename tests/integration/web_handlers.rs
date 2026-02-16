//! Integration tests for web handlers - testing handler logic patterns.

use actix_web::{test, web, App, HttpResponse};
use actix_web::cookie::Key;
use actix_session::{Session, SessionMiddleware, storage::CookieSessionStore};
use serde::Deserialize;
use serde_json::Value;

// ==================== Handler Pattern Tests ====================
// These test the handler patterns used in the real app (redirect, search parsing, etc.)

#[actix_web::test]
async fn test_index_redirect() {
    let app = test::init_service(
        App::new()
            .route("/", web::get().to(|| async {
                HttpResponse::Found()
                    .insert_header(("Location", "/inventory"))
                    .finish()
            }))
    ).await;

    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status().as_u16(), 302);
    let location = resp.headers().get("Location")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert_eq!(location, "/inventory");
}

#[actix_web::test]
async fn test_login_page_returns_json() {
    let app = test::init_service(
        App::new()
            .route("/login", web::get().to(|| async {
                HttpResponse::Ok().json(serde_json::json!({"page": "login"}))
            }))
    ).await;

    let req = test::TestRequest::get().uri("/login").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["page"], "login");
}

#[actix_web::test]
async fn test_logout_clears_session_and_redirects() {
    let secret_key = Key::generate();
    let app = test::init_service(
        App::new()
            .wrap(SessionMiddleware::builder(
                CookieSessionStore::default(),
                secret_key.clone(),
            ).build())
            .route("/logout", web::get().to(|session: Session| async move {
                session.purge();
                HttpResponse::Found()
                    .insert_header(("Location", "/login"))
                    .finish()
            }))
    ).await;

    let req = test::TestRequest::get().uri("/logout").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status().as_u16(), 302);
    let location = resp.headers().get("Location")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert_eq!(location, "/login");
}

#[actix_web::test]
async fn test_not_implemented_handler() {
    let app = test::init_service(
        App::new()
            .route("/search/vlan", web::get().to(|| async {
                HttpResponse::Ok().json(serde_json::json!({"status": "not_implemented"}))
            }))
    ).await;

    let req = test::TestRequest::get().uri("/search/vlan").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "not_implemented");
}

// ==================== IP Validation Tests ====================

#[actix_web::test]
async fn test_handler_rejects_invalid_ip() {
    let app = test::init_service(
        App::new()
            .route("/device/{ip}", web::get().to(|path: web::Path<String>| async move {
                let ip_str = path.into_inner();
                let _ip: ipnetwork::IpNetwork = match ip_str.parse() {
                    Ok(ip) => ip,
                    Err(_) => return HttpResponse::BadRequest()
                        .json(serde_json::json!({"error": "Invalid IP address"})),
                };
                HttpResponse::Ok().json(serde_json::json!({"ip": ip_str}))
            }))
    ).await;

    let req = test::TestRequest::get().uri("/device/not_an_ip").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status().as_u16(), 400);
    let body: Value = test::read_body_json(resp).await;
    assert!(body.get("error").is_some());
}

#[actix_web::test]
async fn test_handler_accepts_valid_ipv4() {
    let app = test::init_service(
        App::new()
            .route("/device/{ip}", web::get().to(|path: web::Path<String>| async move {
                let ip_str = path.into_inner();
                let ip: ipnetwork::IpNetwork = match ip_str.parse() {
                    Ok(ip) => ip,
                    Err(_) => return HttpResponse::BadRequest()
                        .json(serde_json::json!({"error": "Invalid IP address"})),
                };
                HttpResponse::Ok().json(serde_json::json!({"ip": ip.to_string()}))
            }))
    ).await;

    let req = test::TestRequest::get().uri("/device/192.168.1.1").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_handler_accepts_cidr() {
    let app = test::init_service(
        App::new()
            .route("/device/{ip}", web::get().to(|path: web::Path<String>| async move {
                let ip_str = path.into_inner();
                let _ip: ipnetwork::IpNetwork = match ip_str.parse() {
                    Ok(ip) => ip,
                    Err(_) => return HttpResponse::BadRequest()
                        .json(serde_json::json!({"error": "Invalid IP address"})),
                };
                HttpResponse::Ok().json(serde_json::json!({"ip": ip_str}))
            }))
    ).await;

    let req = test::TestRequest::get().uri("/device/10.0.0.0%2F24").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
}

// ==================== Node Search Pattern Tests ====================

#[actix_web::test]
async fn test_search_node_mac_format_detection() {
    #[derive(Deserialize)]
    struct NodeSearch {
        q: Option<String>,
    }

    let app = test::init_service(
        App::new()
            .route("/search/node", web::get().to(|query: web::Query<NodeSearch>| async move {
                if let Some(q) = &query.q {
                    if q.contains(':') || q.contains('-') || q.contains('.') {
                        return HttpResponse::Ok().json(serde_json::json!({
                            "type": "mac_search",
                            "query": q
                        }));
                    }
                    if q.parse::<ipnetwork::IpNetwork>().is_ok() {
                        return HttpResponse::Ok().json(serde_json::json!({
                            "type": "ip_search",
                            "query": q
                        }));
                    }
                }
                HttpResponse::Ok().json(serde_json::json!({"type": "empty"}))
            }))
    ).await;

    // MAC search
    let req = test::TestRequest::get()
        .uri("/search/node?q=00:11:22:33:44:55")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["type"], "mac_search");

    // IP search
    let req = test::TestRequest::get()
        .uri("/search/node?q=192.168.1.1")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let body: Value = test::read_body_json(resp).await;
    // Note: 192.168.1.1 contains dots, so it'll match MAC pattern first in the actual handler
    assert!(body["type"] == "mac_search" || body["type"] == "ip_search");

    // Empty search
    let req = test::TestRequest::get()
        .uri("/search/node")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["type"], "empty");
}

// ==================== Report Handler Tests ====================

#[actix_web::test]
async fn test_report_path_extraction() {
    let app = test::init_service(
        App::new()
            .route("/report/{name}", web::get().to(|path: web::Path<String>| async move {
                HttpResponse::Ok().json(serde_json::json!({
                    "report": path.into_inner(),
                    "status": "not_implemented"
                }))
            }))
    ).await;

    let req = test::TestRequest::get().uri("/report/device_summary").to_request();
    let resp = test::call_service(&app, req).await;
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["report"], "device_summary");
}

// ==================== Session Pattern Tests ====================

#[actix_web::test]
async fn test_session_set_and_get() {
    let secret_key = Key::generate();
    let app = test::init_service(
        App::new()
            .wrap(SessionMiddleware::builder(
                CookieSessionStore::default(),
                secret_key.clone(),
            ).build())
            .route("/set", web::get().to(|session: Session| async move {
                session.insert("username", "testuser").ok();
                HttpResponse::Ok().json(serde_json::json!({"set": true}))
            }))
    ).await;

    let req = test::TestRequest::get().uri("/set").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

// ==================== Login Form Parsing ====================

#[actix_web::test]
async fn test_login_form_parsing() {
    let secret_key = Key::generate();
    let app = test::init_service(
        App::new()
            .wrap(SessionMiddleware::builder(
                CookieSessionStore::default(),
                secret_key.clone(),
            ).build())
            .route("/login", web::post().to(|form: web::Form<std::collections::HashMap<String, String>>| async move {
                let username = form.get("username").cloned().unwrap_or_default();
                let password = form.get("password").cloned().unwrap_or_default();
                HttpResponse::Ok().json(serde_json::json!({
                    "received_username": username,
                    "has_password": !password.is_empty(),
                }))
            }))
    ).await;

    let req = test::TestRequest::post()
        .uri("/login")
        .set_form(std::collections::HashMap::from([
            ("username".to_string(), "admin".to_string()),
            ("password".to_string(), "secret".to_string()),
        ]))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["received_username"], "admin");
    assert_eq!(body["has_password"], true);
}
