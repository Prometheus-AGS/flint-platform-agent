//! Composition root state.
//!
//! Constructs the four plane adapters from [`GatewayConfig`], assembles
//! [`fpa_app::TaskRunner`] from them, and holds both for injection as Axum
//! shared state. This is the only place concrete adapters are wired.

use crate::config::GatewayConfig;
use crate::jwks::JwksVerifier;
use fpa_app::{TaskRunner, TaskStore};
use fpa_fabric::FabricAdapter;
use fpa_forge::ForgeAdapter;
use fpa_gate::GateAdapter;
use fpa_mcp::McpClientAdapter;
use std::sync::Arc;

/// Shared application state available to every handler.
pub struct AppState {
    /// The use-case runner, wired to the four plane ports.
    pub runner: Arc<TaskRunner>,
    /// Resolved configuration.
    pub config: Arc<GatewayConfig>,
    /// In-memory store of submitted tasks (A2A status/cancel).
    pub tasks: Arc<TaskStore>,
    /// JWKS verifier for directly-received tokens (`None` if no JWKS configured).
    pub jwks: Option<Arc<JwksVerifier>>,
}

impl AppState {
    /// Build state by constructing the plane adapters from config and wiring
    /// them into a [`TaskRunner`].
    #[must_use]
    pub fn new(config: GatewayConfig) -> Self {
        let forge = Arc::new(ForgeAdapter::new(config.forge_url.clone()));
        let fabric = Arc::new(FabricAdapter::new(config.fabric_endpoint.clone()));
        let gate = Arc::new(GateAdapter::new(config.gate_admin_url.clone()));
        let mcp = Arc::new(McpClientAdapter::new(
            // No downstream MCP server configured yet; placeholder endpoint.
            // Multi-server composition is a carry-forward.
            String::new(),
        ));

        let jwks = config.jwks_url.clone().map(|url| {
            Arc::new(JwksVerifier::new(
                url,
                config.jwt_issuers.clone(),
                config.jwt_audiences.clone(),
            ))
        });

        let runner = Arc::new(TaskRunner::new(forge, fabric, gate, mcp));
        Self {
            runner,
            config: Arc::new(config),
            tasks: Arc::new(TaskStore::default()),
            jwks,
        }
    }
}
