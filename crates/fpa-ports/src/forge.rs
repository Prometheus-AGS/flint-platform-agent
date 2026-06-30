//! Port: Flint Forge metadata & data management.
//!
//! Source of truth for what the fabric contains (tables, schema, components).
//! Implemented by `fpa-forge`.

use crate::error::PortError;
use async_trait::async_trait;

/// Read access to forge fabric metadata and Quarry-driven inspection.
#[async_trait]
pub trait ForgeMetadata: Send + Sync {
    /// List the table metadata visible to the current operator context.
    async fn list_tables(&self) -> Result<serde_json::Value, PortError>;

    /// Fetch detailed metadata for a single entity by name.
    async fn describe_table(&self, name: &str) -> Result<serde_json::Value, PortError>;
}
