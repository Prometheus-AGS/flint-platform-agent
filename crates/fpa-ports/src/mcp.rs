//! Port: downstream MCP client.
//!
//! The agent is an MCP client that composes other servers' tools into its
//! administrative toolset. Implemented by `fpa-mcp`.

use crate::error::PortError;
use async_trait::async_trait;

/// Client access to a downstream MCP server.
#[async_trait]
pub trait McpClient: Send + Sync {
    /// List the tools advertised by the downstream MCP server.
    async fn list_tools(&self) -> Result<serde_json::Value, PortError>;

    /// Invoke a downstream tool by name with the given arguments.
    async fn call_tool(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, PortError>;
}
