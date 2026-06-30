//! MCP server surface — **HTTP-Streaming transport only** (no stdio).
//!
//! Implements the MCP protocol as JSON-RPC 2.0 over `POST /mcp`. This is the
//! agent's MCP **server** role: it exposes the fabric's administrative tools to
//! any MCP host. (The agent's MCP **client** role lives in the `fpa-mcp` adapter,
//! not here.)
//!
//! Stub: parses the JSON-RPC envelope and dispatches the core methods
//! (`initialize`, `tools/list`, `tools/call`) with placeholder results.

use axum::{Json, Router, routing::post};
use serde::{Deserialize, Serialize};

/// JSON-RPC protocol version string.
const JSONRPC_VERSION: &str = "2.0";
/// JSON-RPC error code for an unrecognized method.
const METHOD_NOT_FOUND: i64 = -32601;

/// Routes for the MCP server surface.
pub fn router() -> Router<std::sync::Arc<crate::state::AppState>> {
    Router::new().route("/mcp", post(rpc))
}

/// A JSON-RPC 2.0 request envelope.
#[derive(Debug, Deserialize)]
struct RpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    /// Present for calls, absent for notifications.
    id: Option<serde_json::Value>,
    method: String,
    #[serde(default)]
    #[allow(dead_code)]
    params: serde_json::Value,
}

/// A JSON-RPC 2.0 response envelope (success or error, never both).
#[derive(Debug, Serialize)]
struct RpcResponse {
    jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<RpcError>,
}

/// A JSON-RPC 2.0 error object.
#[derive(Debug, Serialize)]
struct RpcError {
    code: i64,
    message: String,
}

impl RpcResponse {
    fn ok(id: Option<serde_json::Value>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION,
            id,
            result: Some(result),
            error: None,
        }
    }

    fn err(id: Option<serde_json::Value>, code: i64, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION,
            id,
            result: None,
            error: Some(RpcError {
                code,
                message: message.into(),
            }),
        }
    }
}

/// `POST /mcp` — JSON-RPC 2.0 entry point.
///
/// Stub: dispatches the core MCP methods. The real handler wires `tools/list`
/// and `tools/call` to the fabric administrative tool catalog and streams large
/// results back over the HTTP-Streaming transport.
async fn rpc(Json(req): Json<RpcRequest>) -> Json<RpcResponse> {
    let response = match req.method.as_str() {
        "initialize" => RpcResponse::ok(
            req.id,
            serde_json::json!({
                "protocolVersion": "2025-06-18",
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "flint-platform-agent", "version": env!("CARGO_PKG_VERSION") }
            }),
        ),
        "tools/list" => RpcResponse::ok(req.id, serde_json::json!({ "tools": [] })),
        "tools/call" => {
            RpcResponse::err(req.id, METHOD_NOT_FOUND, "tools/call not yet implemented")
        }
        other => RpcResponse::err(req.id, METHOD_NOT_FOUND, format!("unknown method: {other}")),
    };
    Json(response)
}
