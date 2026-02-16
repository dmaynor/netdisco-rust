//! Authentication middleware and helpers.

use actix_web::dev::ServiceRequest;
use actix_web::HttpResponse;
use actix_session::Session;

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

use actix_session::SessionExt;

/// Check if the session (from HttpRequest) is authenticated.
pub fn session_is_authenticated(session: &Session) -> bool {
    session.get::<String>("username").ok().flatten().is_some()
}

/// Check if the session user is an admin.
pub fn session_is_admin(session: &Session) -> bool {
    session.get::<bool>("admin").ok().flatten().unwrap_or(false)
}

/// Helper: return 401 if not authenticated, checking the no_auth config flag.
pub fn require_auth(session: &Session, config: &crate::config::NetdiscoConfig) -> Option<HttpResponse> {
    if config.no_auth {
        return None; // Auth disabled
    }
    if !session_is_authenticated(session) {
        Some(HttpResponse::Unauthorized().json(serde_json::json!({"error": "Authentication required"})))
    } else {
        None
    }
}

/// Helper: return 403 if not admin.
pub fn require_admin(session: &Session, config: &crate::config::NetdiscoConfig) -> Option<HttpResponse> {
    if let Some(resp) = require_auth(session, config) {
        return Some(resp);
    }
    if !config.no_auth && !session_is_admin(session) {
        Some(HttpResponse::Forbidden().json(serde_json::json!({"error": "Admin access required"})))
    } else {
        None
    }
}

/// Allowed job action types for validation.
const VALID_JOB_ACTIONS: &[&str] = &[
    "discover", "discoverall", "macsuck", "macwalk",
    "arpnip", "arpwalk", "nbtstat", "nbtwalk",
    "expire", "delete", "portcontrol", "portname",
    "portvlan", "power", "graph", "show", "stats",
];

/// Validate that a job action is in the allowed list.
pub fn is_valid_job_action(action: &str) -> bool {
    VALID_JOB_ACTIONS.contains(&action)
}
