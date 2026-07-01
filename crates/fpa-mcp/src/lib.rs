//! `fpa-mcp` — adapter implementing [`fpa_ports::McpClient`].
//!
//! The agent's MCP **client** role: composes downstream MCP servers' tools into
//! its administrative toolset via JSON-RPC 2.0 over HTTP (the same wire the
//! agent's own MCP server speaks — see the `mcp-server` house pattern). Uses
//! `reqwest`; no heavyweight MCP SDK.
//!
//! The agent's MCP **server** surface lives in `fpa-gateway` (`routes/mcp.rs`),
//! not here — this crate is exclusively the outbound client adapter.

use async_trait::async_trait;
use fpa_ports::{McpClient, PortError};
use serde_json::{Value, json};

/// Client adapter for a single downstream MCP server (JSON-RPC over HTTP).
pub struct McpClientAdapter {
    endpoint: String,
    http: reqwest::Client,
}

impl McpClientAdapter {
    /// Construct an adapter pointed at a downstream MCP server endpoint.
    #[must_use]
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            http: reqwest::Client::new(),
        }
    }

    /// Send a JSON-RPC 2.0 request and return the `result`, mapping transport and
    /// protocol failures onto [`PortError`].
    async fn rpc(&self, method: &str, params: Value) -> Result<Value, PortError> {
        if self.endpoint.is_empty() {
            return Err(PortError::Transport(
                "no downstream MCP endpoint configured".to_owned(),
            ));
        }
        let body = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
        let resp = self
            .http
            .post(&self.endpoint)
            .json(&body)
            .send()
            .await
            .map_err(|e| PortError::Transport(e.to_string()))?;
        let envelope: Value = resp
            .json()
            .await
            .map_err(|e| PortError::Decode(e.to_string()))?;

        if let Some(err) = envelope.get("error").filter(|e| !e.is_null()) {
            return Err(PortError::Downstream(err.to_string()));
        }
        Ok(envelope.get("result").cloned().unwrap_or(Value::Null))
    }
}

#[async_trait]
impl McpClient for McpClientAdapter {
    async fn list_tools(&self) -> Result<Value, PortError> {
        self.rpc("tools/list", Value::Null).await
    }

    async fn call_tool(&self, name: &str, arguments: Value) -> Result<Value, PortError> {
        self.rpc(
            "tools/call",
            json!({ "name": name, "arguments": arguments }),
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn empty_endpoint_is_transport_error() {
        let client = McpClientAdapter::new("");
        let err = client.list_tools().await.unwrap_err();
        assert!(matches!(err, PortError::Transport(_)));
    }
}
