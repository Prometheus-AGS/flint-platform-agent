//! Port: Flint Forge metadata & data management.
//!
//! Source of truth for what the fabric contains (tables, schema, components).
//! Implemented by `fpa-forge`.

use crate::error::PortError;
use async_trait::async_trait;

/// Read access to forge fabric metadata and Quarry-driven inspection.
///
/// Methods take the operator's `bearer` (the gate-issued JWT). Metadata reads may
/// ignore it (forge's OpenAPI is public); data reads forward it as
/// `Authorization: Bearer` so forge applies RLS. The adapter never fabricates a
/// credential — a `None` bearer means "no operator identity".
#[async_trait]
pub trait ForgeMetadata: Send + Sync {
    /// List the table metadata visible to the operator.
    async fn list_tables(&self, bearer: Option<&str>) -> Result<serde_json::Value, PortError>;

    /// Fetch detailed metadata for a single entity by name.
    async fn describe_table(
        &self,
        name: &str,
        bearer: Option<&str>,
    ) -> Result<serde_json::Value, PortError>;

    /// Create a row in `schema`.`table` via forge's REST insert under the operator
    /// bearer. Forge's REST route is `POST /{schema}/{table}` merged at the gateway
    /// root (no `/rest` prefix). Forge (RLS + Keto/Cedar) is the authorization
    /// authority.
    ///
    /// A `None` bearer is rejected (`Unauthorized`) — writes always require the
    /// operator's identity.
    async fn create_entity(
        &self,
        schema: &str,
        table: &str,
        object: serde_json::Value,
        bearer: Option<&str>,
    ) -> Result<serde_json::Value, PortError>;
}
