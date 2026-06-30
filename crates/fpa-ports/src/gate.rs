//! Port: Flint Gate admin API.
//!
//! Administers routes, auth providers, and streaming config via gate's admin
//! server (`:4457`, never public). Implemented by `fpa-gate`.

use crate::error::PortError;
use async_trait::async_trait;

/// Administrative access to Flint Gate.
#[async_trait]
pub trait GateAdmin: Send + Sync {
    /// List the routes currently configured in gate.
    async fn list_routes(&self) -> Result<serde_json::Value, PortError>;
}
