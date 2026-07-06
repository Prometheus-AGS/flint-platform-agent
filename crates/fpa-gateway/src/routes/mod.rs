//! HTTP route surfaces for the agent gateway.
//!
//! One module per protocol surface, each kept well under the 500-line limit:
//! - [`agui`]   — AG-UI Server-Sent Events stream (agent → UI)
//! - [`a2a`]    — A2A (Agent2Agent) administrative task endpoints
//! - [`mcp`]    — MCP server surface (HTTP-Streaming only)
//! - [`fabric`] — fabric realtime subscribe → SSE bridge
//!
//! Each surface exposes a `router()` returning an [`axum::Router`]; the
//! composition root in `main.rs` merges them and supplies shared state.

pub mod a2a;
pub mod agui;
pub mod fabric;
pub mod mcp;

/// Liveness probe.
pub async fn healthz() -> &'static str {
    "ok"
}
