//! Composition root state.
//!
//! Constructs the four plane adapters from [`GatewayConfig`], assembles
//! [`fpa_app::TaskRunner`] from them, and holds both for injection as Axum
//! shared state. This is the only place concrete adapters are wired.

use crate::config::GatewayConfig;
use crate::jwks::JwksVerifier;
use fpa_app::{InMemoryProjectStore, TaskRunner, TaskStore};
use fpa_fabric::FabricAdapter;
use fpa_forge::ForgeAdapter;
use fpa_gate::GateAdapter;
use fpa_mcp::McpClientAdapter;
use fpa_ports::ProjectStore;
use fpa_store_pg::PgProjectStore;
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
    ///
    /// Async because the durable `ProjectStore` (p6-c001) connects to Postgres at
    /// startup when `FPA_PROJECT_DB_URL` is configured.
    pub async fn new(config: GatewayConfig) -> Self {
        let mut forge_adapter = ForgeAdapter::new(config.forge_url.clone());
        if let Some(prefix) = config.forge_rest_prefix.clone() {
            forge_adapter = forge_adapter.with_rest_prefix(prefix);
        }
        let forge = Arc::new(forge_adapter);
        let fabric = Arc::new(FabricAdapter::new(config.fabric_endpoint.clone()));
        let gate = Arc::new(
            GateAdapter::new(config.gate_admin_url.clone())
                .with_admin_token(config.gate_admin_token.clone()),
        );
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

        // Agent-owned Project persistence (p6-c001). Durable Postgres store when
        // `FPA_PROJECT_DB_URL` is set; in-memory otherwise. A bad/unreachable URL
        // logs and falls back to in-memory rather than crashing the agent.
        let projects = select_project_store(config.project_db_url.as_deref()).await;

        let runner = Arc::new(TaskRunner::new(forge, fabric, gate, mcp, projects));
        Self {
            runner,
            config: Arc::new(config),
            tasks: Arc::new(TaskStore::default()),
            jwks,
        }
    }
}

/// Choose the `ProjectStore`: durable Postgres when a URL is configured and
/// reachable, else the in-memory store. A connect failure is logged (never with the
/// URL) and degrades to in-memory — a bad DB config must not take the agent down.
async fn select_project_store(db_url: Option<&str>) -> Arc<dyn ProjectStore> {
    match db_url {
        None => Arc::new(InMemoryProjectStore::new()),
        Some(url) => match PgProjectStore::connect(url).await {
            Ok(store) => {
                tracing::info!("project store: durable Postgres");
                Arc::new(store)
            }
            Err(e) => {
                // Do NOT log the URL (it carries a password). Log only the error kind.
                tracing::warn!(error = %e, "project DB unreachable — falling back to in-memory store");
                Arc::new(InMemoryProjectStore::new())
            }
        },
    }
}
