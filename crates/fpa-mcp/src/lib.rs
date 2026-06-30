//! `fpa-mcp` — adapter implementing [`fpa_ports::McpClient`].
//!
//! The agent's MCP **client** role: composes downstream MCP servers' tools into
//! its administrative toolset.
//!
//! Note: the agent's MCP **server** surface (HTTP-Streaming only) and **app**
//! surface live in the interface crate (`fpa-gateway`), not here — this crate is
//! exclusively the outbound client adapter.

use async_trait::async_trait;
use fpa_ports::{McpClient, PortError};

/// Client adapter for a single downstream MCP server.
pub struct McpClientAdapter {
    /// Endpoint of the downstream MCP server.
    pub endpoint: String,
}

impl McpClientAdapter {
    /// Construct an adapter pointed at a downstream MCP server endpoint.
    #[must_use]
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }
}

#[async_trait]
impl McpClient for McpClientAdapter {
    async fn list_tools(&self) -> Result<serde_json::Value, PortError> {
        todo!("list tools from MCP server {}", self.endpoint)
    }

    async fn call_tool(
        &self,
        _name: &str,
        _arguments: serde_json::Value,
    ) -> Result<serde_json::Value, PortError> {
        todo!("call downstream MCP tool")
    }
}
