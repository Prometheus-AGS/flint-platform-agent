//! `fpa-gateway` — the Axum composition root for the Flint Platform Agent.
//!
//! This is the **only** crate that imports concrete adapters. It wires the four
//! plane adapters into the app-layer `TaskRunner`, then mounts the protocol
//! surfaces:
//!
//! - AG-UI event stream (agent → UI)
//! - A2A task endpoints
//! - A2UI component emission
//! - MCP **server** (HTTP-Streaming only) and **app** surfaces
//!
//! `anyhow` is permitted here (binary entry point); library crates use `thiserror`.

use anyhow::Context as _;
use axum::{Router, routing::get};
use std::net::SocketAddr;

mod routes;

/// Default bind address. Override with `FPA_GATEWAY_ADDR` (e.g. `0.0.0.0:9090`).
const DEFAULT_ADDR: &str = "0.0.0.0:8088";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let app = build_router();

    let addr: SocketAddr = std::env::var("FPA_GATEWAY_ADDR")
        .unwrap_or_else(|_| DEFAULT_ADDR.to_owned())
        .parse()
        .context("FPA_GATEWAY_ADDR must be a valid socket address (host:port)")?;
    tracing::info!(%addr, "fpa-gateway listening");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;
    axum::serve(listener, app).await?;
    Ok(())
}

/// Build the agent router by merging each protocol surface.
///
/// Adapter wiring (the four plane adapters → `fpa_app::TaskRunner`) is injected
/// as shared state once the handlers consume it; the stubs are stateless for now.
fn build_router() -> Router {
    Router::new()
        .route("/healthz", get(routes::healthz))
        .merge(routes::agui::router())
        .merge(routes::a2a::router())
        .merge(routes::mcp::router())
}
