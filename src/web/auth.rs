//! Authentication middleware and helpers.

use actix_web::{dev::ServiceRequest, Error, HttpResponse};
use actix_session::SessionExt;

/// Check if the current request is authenticated.
pub fn is_authenticated(req: &ServiceRequest) -> bool {
    let session = req.get_session();
    session.get::<String>("username").ok().flatten().is_some()
}

/// Check if the current user has admin role.
pub fn is_admin(req: &ServiceRequest) -> bool {
    let session = req.get_session();
    session.get::<bool>("admin").ok().flatten().unwrap_or(false)
}

/// Get the current username from the session.
pub fn current_user(req: &ServiceRequest) -> Option<String> {
    let session = req.get_session();
    session.get::<String>("username").ok().flatten()
}
