//! Database access layer using SQLx with PostgreSQL.
//!
//! Provides connection pooling, migrations, and CRUD operations
//! for all Netdisco models.

pub mod pool;
pub mod queries;
pub mod migrate;

pub use pool::*;
pub use queries::*;
pub use migrate::*;
