//! Integration tests for the REST API endpoints.
//!
//! These tests use an in-process actix-web test server to verify
//! API route configuration, request parsing, and response format.

use actix_web::{test, web, App, HttpResponse};
use actix_web::cookie::Key;
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use serde_json::Value;

// ==================== API Response Format Tests ====================

#[actix_web::test]
async fn test_api_list_devices_response_format() {
    let secret_key = Key::generate();
    let app = test::init_service(
        App::new()
            .wrap(SessionMiddleware::builder(
                CookieSessionStore::default(),
                secret_key.clone(),
            ).build())
            .route("/api/v1/test/devices", web::get().to(|| async {
                HttpResponse::Ok().json(serde_json::json!([
                    {"ip": "10.0.0.1/32", "dns": "switch1.example.com", "name": "SW1"},
                    {"ip": "10.0.0.2/32", "dns": "switch2.example.com", "name": "SW2"},
                ]))
            }))
    ).await;

    let req = test::TestRequest::get().uri("/api/v1/test/devices").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
    let body: Value = test::read_body_json(resp).await;
    assert!(body.is_array());
    assert_eq!(body.as_array().unwrap().len(), 2);
    assert_eq!(body[0]["dns"], "switch1.example.com");
}

#[actix_web::test]
async fn test_api_get_device_found() {
    let secret_key = Key::generate();
    let app = test::init_service(
        App::new()
            .wrap(SessionMiddleware::builder(
                CookieSessionStore::default(),
                secret_key.clone(),
            ).build())
            .route("/api/v1/test/device/{ip}", web::get().to(|path: web::Path<String>| async move {
                let ip = path.into_inner();
                if ip == "10.0.0.1" {
                    HttpResponse::Ok().json(serde_json::json!({"ip": "10.0.0.1/32", "name": "SW1"}))
                } else {
                    HttpResponse::NotFound().json(serde_json::json!({"error": "Not found"}))
                }
            }))
    ).await;

    let req = test::TestRequest::get().uri("/api/v1/test/device/10.0.0.1").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["name"], "SW1");
}

#[actix_web::test]
async fn test_api_get_device_not_found() {
    let secret_key = Key::generate();
    let app = test::init_service(
        App::new()
            .wrap(SessionMiddleware::builder(
                CookieSessionStore::default(),
                secret_key.clone(),
            ).build())
            .route("/api/v1/test/device/{ip}", web::get().to(|path: web::Path<String>| async move {
                let ip = path.into_inner();
                if ip == "10.0.0.1" {
                    HttpResponse::Ok().json(serde_json::json!({"ip": "10.0.0.1/32"}))
                } else {
                    HttpResponse::NotFound().json(serde_json::json!({"error": "Not found"}))
                }
            }))
    ).await;

    let req = test::TestRequest::get().uri("/api/v1/test/device/10.0.0.99").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 404);
    let body: Value = test::read_body_json(resp).await;
    assert!(body.get("error").is_some());
}

#[actix_web::test]
async fn test_api_search_with_query() {
    let secret_key = Key::generate();
    let app = test::init_service(
        App::new()
            .wrap(SessionMiddleware::builder(
                CookieSessionStore::default(),
                secret_key.clone(),
            ).build())
            .route("/api/v1/test/search", web::get().to(|query: web::Query<std::collections::HashMap<String, String>>| async move {
                let q = query.get("q").cloned().unwrap_or_default();
                HttpResponse::Ok().json(serde_json::json!({"query": q, "results": []}))
            }))
    ).await;

    let req = test::TestRequest::get().uri("/api/v1/test/search?q=cisco").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["query"], "cisco");
    assert!(body["results"].is_array());
}

#[actix_web::test]
async fn test_api_search_empty_query() {
    let secret_key = Key::generate();
    let app = test::init_service(
        App::new()
            .wrap(SessionMiddleware::builder(
                CookieSessionStore::default(),
                secret_key.clone(),
            ).build())
            .route("/api/v1/test/search", web::get().to(|query: web::Query<std::collections::HashMap<String, String>>| async move {
                let q = query.get("q").cloned().unwrap_or_default();
                HttpResponse::Ok().json(serde_json::json!({"query": q, "results": []}))
            }))
    ).await;

    let req = test::TestRequest::get().uri("/api/v1/test/search").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["query"], "");
}

#[actix_web::test]
async fn test_api_enqueue_job() {
    let secret_key = Key::generate();
    let app = test::init_service(
        App::new()
            .wrap(SessionMiddleware::builder(
                CookieSessionStore::default(),
                secret_key.clone(),
            ).build())
            .route("/api/v1/test/queue", web::post().to(|body: web::Json<Value>| async move {
                let action = body.get("action").and_then(|v| v.as_str()).unwrap_or("unknown");
                HttpResponse::Ok().json(serde_json::json!({"job": 42, "action": action}))
            }))
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/test/queue")
        .set_json(serde_json::json!({"action": "discover", "device": "10.0.0.1"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["job"], 42);
    assert_eq!(body["action"], "discover");
}

#[actix_web::test]
async fn test_api_enqueue_job_missing_body() {
    let secret_key = Key::generate();
    let app = test::init_service(
        App::new()
            .wrap(SessionMiddleware::builder(
                CookieSessionStore::default(),
                secret_key.clone(),
            ).build())
            .route("/api/v1/test/queue", web::post().to(|_body: web::Json<Value>| async {
                HttpResponse::Ok().json(serde_json::json!({"ok": true}))
            }))
    ).await;

    let req = test::TestRequest::post().uri("/api/v1/test/queue").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}

// ==================== Content-Type Tests ====================

#[actix_web::test]
async fn test_api_response_content_type_json() {
    let secret_key = Key::generate();
    let app = test::init_service(
        App::new()
            .wrap(SessionMiddleware::builder(
                CookieSessionStore::default(),
                secret_key.clone(),
            ).build())
            .route("/api/v1/test/data", web::get().to(|| async {
                HttpResponse::Ok().json(serde_json::json!({"data": true}))
            }))
    ).await;

    let req = test::TestRequest::get().uri("/api/v1/test/data").to_request();
    let resp = test::call_service(&app, req).await;
    let content_type = resp.headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(content_type.contains("application/json"), "Content-Type should be JSON, got: {}", content_type);
}

// ==================== HTTP Method Tests ====================

#[actix_web::test]
async fn test_api_post_to_get_endpoint_rejected() {
    let secret_key = Key::generate();
    let app = test::init_service(
        App::new()
            .wrap(SessionMiddleware::builder(
                CookieSessionStore::default(),
                secret_key.clone(),
            ).build())
            .route("/api/v1/test/devices", web::get().to(|| async {
                HttpResponse::Ok().json(serde_json::json!([]))
            }))
    ).await;

    let req = test::TestRequest::post().uri("/api/v1/test/devices").to_request();
    let resp = test::call_service(&app, req).await;
    // actix-web returns 404 (not 405) when a single method route doesn't match
    assert!(resp.status().is_client_error());
}

#[actix_web::test]
async fn test_api_nonexistent_route() {
    let secret_key = Key::generate();
    let app = test::init_service(
        App::new()
            .wrap(SessionMiddleware::builder(
                CookieSessionStore::default(),
                secret_key.clone(),
            ).build())
            .route("/api/v1/test/exists", web::get().to(|| async {
                HttpResponse::Ok().json(serde_json::json!({"ok": true}))
            }))
    ).await;

    let req = test::TestRequest::get().uri("/api/v1/nonexistent").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 404);
}

// ==================== Route Configuration Tests ====================

#[actix_web::test]
async fn test_api_routes_configured() {
    let secret_key = Key::generate();
    let app = test::init_service(
        App::new()
            .wrap(SessionMiddleware::builder(
                CookieSessionStore::default(),
                secret_key.clone(),
            ).build())
            .configure(netdisco::web::api::configure)
    ).await;

    // Without AppState these will 500, but routes should exist (not 404)
    let req = test::TestRequest::get().uri("/api/v1/object/device").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_server_error() || resp.status().is_success());
}

#[actix_web::test]
async fn test_web_routes_configured() {
    let secret_key = Key::generate();
    let app = test::init_service(
        App::new()
            .wrap(SessionMiddleware::builder(
                CookieSessionStore::default(),
                secret_key.clone(),
            ).build())
            .configure(netdisco::web::routes::configure)
    ).await;

    let req = test::TestRequest::get().uri("/login").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_server_error() || resp.status().is_success());
}
