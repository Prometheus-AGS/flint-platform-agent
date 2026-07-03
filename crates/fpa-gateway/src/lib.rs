//! `fpa-gateway` — the Axum composition root for the Flint Platform Agent.
//!
//! This is the **only** crate that imports concrete adapters. It loads config,
//! constructs the four plane adapters into the app-layer `TaskRunner`
//! ([`state::AppState`]), and mounts the protocol surfaces with that shared state:
//!
//! - AG-UI event stream (agent → UI)
//! - A2A task endpoints
//! - MCP **server** (HTTP-Streaming only)
//!
//! The binary (`main.rs`) is a thin entry point over [`build_router`]; the router
//! is exposed here so integration tests can drive the real handler stack.
//!
//! `anyhow` is permitted at the binary entry point; library code uses `thiserror`.

pub mod api_error;
pub mod config;
pub mod identity;
pub mod jwks;
pub mod routes;
pub mod state;

use axum::{Router, routing::get};
use std::sync::Arc;

pub use config::GatewayConfig;
pub use state::AppState;

/// Build the agent router by merging each protocol surface with shared state.
pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/healthz", get(routes::healthz))
        .merge(routes::agui::router())
        .merge(routes::a2a::router())
        .merge(routes::mcp::router())
        .with_state(state)
}
