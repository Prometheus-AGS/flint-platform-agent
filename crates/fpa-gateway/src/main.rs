//! `fpa-gateway` — the Axum composition root for the Flint Platform Agent.
//!
//! This is the **only** crate that imports concrete adapters. It loads config,
//! constructs the four plane adapters into the app-layer `TaskRunner`
//! ([`state::AppState`]), and mounts the protocol surfaces with that shared
//! state:
//!
//! - AG-UI event stream (agent → UI)
//! - A2A task endpoints
//! - MCP **server** (HTTP-Streaming only)
//!
//! `anyhow` is permitted here (binary entry point); library crates use `thiserror`.

use axum::{Router, routing::get};
use std::sync::Arc;

mod api_error;
mod config;
mod identity;
mod routes;
mod state;

use config::GatewayConfig;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = GatewayConfig::from_env()?;
    let addr = config.addr;
    let state = Arc::new(AppState::new(config));

    let app = build_router(state);

    tracing::info!(%addr, "fpa-gateway listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

/// Build the agent router by merging each protocol surface with shared state.
fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/healthz", get(routes::healthz))
        .merge(routes::agui::router())
        .merge(routes::a2a::router())
        .merge(routes::mcp::router())
        .with_state(state)
}
