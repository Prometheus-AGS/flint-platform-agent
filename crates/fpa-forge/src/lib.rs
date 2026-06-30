//! `fpa-forge` — adapter implementing [`fpa_ports::ForgeMetadata`].
//!
//! Talks to Flint Forge's Quarry (REST/GraphQL DB gateway) to read fabric
//! metadata and drive inspection. Forge is the source of truth for what entities
//! exist. Forwards the operator's verified identity; never fabricates claims.

use async_trait::async_trait;
use fpa_ports::{ForgeMetadata, PortError};

/// HTTP client adapter for Flint Forge Quarry.
pub struct ForgeAdapter {
    /// Base URL of the Quarry gateway.
    pub base_url: String,
}

impl ForgeAdapter {
    /// Construct an adapter pointed at a Quarry gateway base URL.
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }
}

#[async_trait]
impl ForgeMetadata for ForgeAdapter {
    async fn list_tables(&self) -> Result<serde_json::Value, PortError> {
        todo!("GET {}/… table metadata", self.base_url)
    }

    async fn describe_table(&self, _name: &str) -> Result<serde_json::Value, PortError> {
        todo!("describe a single table via Quarry")
    }
}
