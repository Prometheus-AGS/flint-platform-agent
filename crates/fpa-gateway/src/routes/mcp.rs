//! MCP server surface — **HTTP-Streaming transport only** (no stdio).
//!
//! Implements the MCP protocol as JSON-RPC 2.0 over `POST /mcp`. This is the
//! agent's MCP **server** role: it exposes the fabric's administrative tools to
//! any MCP host. (The agent's MCP **client** role lives in the `fpa-mcp` adapter,
//! not here.)
//!
//! Dispatches the core methods (`initialize`, `tools/list`, `tools/call`).
//! `tools/list` is generated from the A2A task catalog and `tools/call` routes
//! through the shared `TaskRunner` (same permission + audit path as A2A).

use crate::{identity::OperatorContext, state::AppState};
use axum::{Json, Router, extract::State, routing::post};
use fpa_app::{AuthContext, catalog};
use fpa_domain::{AdminTask, OperatorId, TaskId, TaskState};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// JSON-RPC protocol version string.
const JSONRPC_VERSION: &str = "2.0";
/// JSON-RPC error code for an unrecognized method.
const METHOD_NOT_FOUND: i64 = -32601;
/// JSON-RPC error code for invalid params.
const INVALID_PARAMS: i64 = -32602;

/// Routes for the MCP server surface.
pub fn router() -> Router<Arc<AppState>> {
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

/// `POST /mcp` — JSON-RPC 2.0 entry point (HTTP-Streaming transport).
///
/// `tools/list` and `tools/call` are backed by the A2A task catalog and the
/// shared `TaskRunner`, so MCP tool calls follow the same permission + audit path
/// as the A2A surface.
async fn rpc(
    State(state): State<Arc<AppState>>,
    // Optional: `initialize`/`tools/list` are unauthenticated handshake/discovery;
    // `tools/call` requires the gate identity (enforced in `call_tool`).
    operator: Option<OperatorContext>,
    Json(req): Json<RpcRequest>,
) -> Json<RpcResponse> {
    let id = req.id.clone();
    let response = match req.method.as_str() {
        "initialize" => RpcResponse::ok(
            id,
            serde_json::json!({
                "protocolVersion": "2025-06-18",
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "flint-platform-agent", "version": env!("CARGO_PKG_VERSION") }
            }),
        ),
        "tools/list" => RpcResponse::ok(id, tool_definitions()),
        "tools/call" => match call_tool(&state, operator.as_ref(), &req.params).await {
            Ok(result) => RpcResponse::ok(id, result),
            Err(e) => RpcResponse::err(id, e.code, e.message),
        },
        other => RpcResponse::err(id, METHOD_NOT_FOUND, format!("unknown method: {other}")),
    };
    Json(response)
}

/// `tools/list` payload — one MCP tool per catalogued task kind.
///
/// Guarantees `tools/list` stays in sync with `tools/call` (both read `catalog`).
fn tool_definitions() -> serde_json::Value {
    let tools: Vec<serde_json::Value> = catalog::CATALOG
        .iter()
        .map(|entry| {
            serde_json::json!({
                "name": entry.kind,
                "description": entry.description,
                // Minimal object schema; per-kind input schemas are a carry-forward.
                "inputSchema": { "type": "object" }
            })
        })
        .collect();
    serde_json::json!({ "tools": tools })
}

/// A dispatch error carrying a JSON-RPC code.
struct CallError {
    code: i64,
    message: String,
}

/// Execute a `tools/call` by routing the named tool through the `TaskRunner`.
///
/// Requires the caller's gate identity (a gate JWT on `POST /mcp`); absence is
/// unauthenticated, never an implicit grant.
async fn call_tool(
    state: &Arc<AppState>,
    operator: Option<&OperatorContext>,
    params: &serde_json::Value,
) -> Result<serde_json::Value, CallError> {
    let Some(operator) = operator else {
        return Err(CallError {
            code: INVALID_PARAMS,
            message: "tools/call requires gate identity (Authorization: Bearer)".to_owned(),
        });
    };

    let name = params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or(CallError {
            code: INVALID_PARAMS,
            message: "tools/call requires a 'name'".to_owned(),
        })?;

    if catalog::lookup(name).is_none() {
        return Err(CallError {
            code: METHOD_NOT_FOUND,
            message: format!("unknown tool: {name}"),
        });
    }

    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    let task = AdminTask {
        id: TaskId(Uuid::new_v4()),
        operator: OperatorId(Uuid::new_v4()),
        kind: name.to_owned(),
        input: arguments,
        state: TaskState::Submitted,
    };
    // Real caller identity from the gate JWT on POST /mcp (p2-c001).
    let auth = AuthContext {
        subject: operator.subject.clone(),
        roles: operator.roles.clone(),
        bearer: Some(operator.bearer.clone()),
    };

    match state.runner.run(&task, &auth).await {
        // MCP tool result shape: content array of text blocks.
        Ok(result) => Ok(serde_json::json!({
            "content": [{ "type": "text", "text": result.to_string() }],
            "isError": false
        })),
        Err(e) => Ok(serde_json::json!({
            "content": [{ "type": "text", "text": e.to_string() }],
            "isError": true
        })),
    }
}
