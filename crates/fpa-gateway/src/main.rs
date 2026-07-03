//! `fpa-gateway` binary — thin entry point over the library's [`build_router`].
//!
//! Loads config, constructs [`AppState`], and serves the merged router. All the
//! composition logic lives in the library (`lib.rs`) so integration tests can
//! drive the same router without a listening socket.
//!
//! `anyhow` is permitted here (binary entry point); library crates use `thiserror`.

use fpa_gateway::{AppState, GatewayConfig, build_router};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = GatewayConfig::from_env()?;
    let addr = config.addr;
    let state = Arc::new(AppState::new(config).await);

    let app = build_router(state);

    tracing::info!(%addr, "fpa-gateway listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
