//! `fpa-gate` — adapter implementing [`fpa_ports::GateAdmin`].
//!
//! Administers Flint Gate via its **admin** server (`:4457`). That port must
//! never be exposed publicly; administrative calls go over a trusted path only
//! (see `CLAUDE.md` → gate dual-server model).

use async_trait::async_trait;
use fpa_ports::{GateAdmin, PortError};

/// Admin-API client adapter for Flint Gate.
pub struct GateAdapter {
    /// Base URL of the gate **admin** server (private).
    pub admin_url: String,
}

impl GateAdapter {
    /// Construct an adapter pointed at the gate admin base URL.
    #[must_use]
    pub fn new(admin_url: impl Into<String>) -> Self {
        Self {
            admin_url: admin_url.into(),
        }
    }
}

#[async_trait]
impl GateAdmin for GateAdapter {
    async fn list_routes(&self) -> Result<serde_json::Value, PortError> {
        todo!("GET {}/routes", self.admin_url)
    }
}
