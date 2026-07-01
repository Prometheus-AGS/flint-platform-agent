//! `fpa-fabric` — adapter implementing [`fpa_ports::FabricClient`].
//!
//! Connects to the Flint Realtime Fabric (gRPC / WS) to subscribe to change and
//! agent-event streams. Realtime events are not assumed pre-authorized for an
//! arbitrary viewer (WAL bypasses RLS — see `CLAUDE.md` cross-plane contracts).

use async_trait::async_trait;
use fpa_ports::{FabricClient, PortError};

/// Client adapter for the Flint Realtime Fabric gateway.
pub struct FabricAdapter {
    /// Endpoint of the fabric gateway.
    pub endpoint: String,
}

impl FabricAdapter {
    /// Construct an adapter pointed at a fabric gateway endpoint.
    #[must_use]
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }
}

#[async_trait]
impl FabricClient for FabricAdapter {
    async fn health(&self) -> Result<(), PortError> {
        // Not implemented until the fabric client transport lands; return a
        // handled error rather than panicking the request path.
        Err(PortError::Downstream(format!(
            "fabric.health not implemented (endpoint {})",
            self.endpoint
        )))
    }
}
