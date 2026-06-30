//! Port: Flint Realtime Fabric.
//!
//! Realtime spine — subscribe to change/agent-event streams to surface live
//! fabric activity in AG-UI. Implemented by `fpa-fabric`.

use crate::error::PortError;
use async_trait::async_trait;

/// Realtime fabric access (subscriptions, realtime operations).
#[async_trait]
pub trait FabricClient: Send + Sync {
    /// Report fabric liveness / connection health.
    async fn health(&self) -> Result<(), PortError>;
}
