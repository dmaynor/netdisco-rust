//! Netdisco Test Suite
//!
//! Comprehensive integration and end-to-end tests organized by:
//! - `unit/` - Pure function and struct tests (no I/O)
//! - `integration/` - HTTP handlers, web API, backend logic
//! - `e2e/` - Full workflow simulations

mod unit;
mod integration;
mod e2e;
