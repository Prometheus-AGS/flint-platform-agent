//! `fpa-app` — Layer 1 use-cases.
//!
//! Orchestrates administrative tasks against the fabric **through ports only**.
//! This crate must never depend on an adapter crate (`fpa-forge`, `fpa-fabric`,
//! `fpa-gate`, `fpa-mcp`) — only on `fpa-ports`, `fpa-domain`, `fpa-protocol`.

pub mod auth;
pub mod catalog;
pub mod error;
pub mod task_runner;

pub use auth::AuthContext;
pub use catalog::{CatalogEntry, TargetPort, kinds, lookup};
pub use error::AppError;
pub use task_runner::TaskRunner;
