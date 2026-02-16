//! Web route configuration.

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/", web::get().to(super::handlers::index))
        .route("/inventory", web::get().to(super::handlers::inventory))
        .route("/device", web::get().to(super::handlers::device_search))
        .route("/device/{ip}", web::get().to(super::handlers::device_detail))
        .route("/device/{ip}/ports", web::get().to(super::handlers::device_ports))
        .route("/device/{ip}/modules", web::get().to(super::handlers::device_modules))
        .route("/device/{ip}/neighbors", web::get().to(super::handlers::device_neighbors))
        .route("/device/{ip}/addresses", web::get().to(super::handlers::device_addresses))
        .route("/device/{ip}/vlans", web::get().to(super::handlers::device_vlans))
        .route("/search/node", web::get().to(super::handlers::search_node))
        .route("/search/device", web::get().to(super::handlers::search_device))
        .route("/search/vlan", web::get().to(super::handlers::search_vlan))
        .route("/search/port", web::get().to(super::handlers::search_port))
        .route("/report/{name}", web::get().to(super::handlers::report))
        .route("/admin/jobqueue", web::get().to(super::handlers::admin_job_queue))
        .route("/admin/users", web::get().to(super::handlers::admin_users))
        .route("/login", web::get().to(super::handlers::login_page))
        .route("/login", web::post().to(super::handlers::login_submit))
        .route("/logout", web::get().to(super::handlers::logout))
        .route("/password", web::post().to(super::handlers::change_password));
}
