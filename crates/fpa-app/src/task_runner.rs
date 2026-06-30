//! The A2A task runner use-case.
//!
//! Resolves a task `kind` to a fabric operation and executes it through the
//! relevant port(s), enforcing gate-derived permissions and emitting an audit
//! record. Holds ports as trait objects so the composition root can inject any
//! adapter (and tests can inject fakes).

use crate::auth::AuthContext;
use crate::catalog::{self, TargetPort};
use crate::error::AppError;
use fpa_domain::AdminTask;
use fpa_ports::{FabricClient, ForgeMetadata, GateAdmin, McpClient};
use std::sync::Arc;

/// Executes administrative tasks against the fabric through injected ports.
pub struct TaskRunner {
    pub forge: Arc<dyn ForgeMetadata>,
    pub fabric: Arc<dyn FabricClient>,
    pub gate: Arc<dyn GateAdmin>,
    pub mcp: Arc<dyn McpClient>,
}

impl TaskRunner {
    /// Construct a runner from the four plane ports.
    #[must_use]
    pub fn new(
        forge: Arc<dyn ForgeMetadata>,
        fabric: Arc<dyn FabricClient>,
        gate: Arc<dyn GateAdmin>,
        mcp: Arc<dyn McpClient>,
    ) -> Self {
        Self {
            forge,
            fabric,
            gate,
            mcp,
        }
    }

    /// Validate, authorize, and execute an administrative task.
    ///
    /// Pipeline: catalog lookup → permission check → port dispatch → audit.
    /// A permission denial returns [`AppError::Unauthorized`] and **never** calls
    /// a port.
    ///
    /// # Errors
    /// - [`AppError::UnknownTaskKind`] if `task.kind` is not catalogued.
    /// - [`AppError::Unauthorized`] if the operator lacks the required role.
    /// - [`AppError::Port`] if the downstream plane fails.
    // Skip `task` and `auth` from auto-capture (their Debug could include input
    // payloads / roles); record only the safe `kind` + subject explicitly.
    #[tracing::instrument(skip_all, fields(task.kind = %task.kind, operator = %auth.subject))]
    pub async fn run(
        &self,
        task: &AdminTask,
        auth: &AuthContext,
    ) -> Result<serde_json::Value, AppError> {
        let entry = catalog::lookup(&task.kind)
            .ok_or_else(|| AppError::UnknownTaskKind(task.kind.clone()))?;

        // Permission enforcement BEFORE any port call (Base Rule 33).
        if !auth.has_role(entry.required_role) {
            // Audit the denial; never log claims/secrets — only ids + decision.
            tracing::warn!(
                operator = %auth.subject,
                kind = entry.kind,
                required_role = entry.required_role,
                decision = "denied",
                "task authorization denied"
            );
            return Err(AppError::Unauthorized(format!(
                "task '{}' requires role '{}'",
                entry.kind, entry.required_role
            )));
        }

        tracing::info!(operator = %auth.subject, kind = entry.kind, decision = "allowed", "task dispatch");

        // Dispatch to the mapped plane. Concrete per-kind argument mapping lands
        // as the ports gain real implementations; here we route by target so the
        // contract + permission gate are exercised end-to-end.
        let outcome = match entry.target {
            TargetPort::Forge => self.dispatch_forge(entry.kind).await,
            TargetPort::Fabric => self
                .fabric
                .health()
                .await
                .map(|()| serde_json::json!({"ok": true})),
            TargetPort::Gate => self.gate.list_routes().await,
            TargetPort::Mcp => self.mcp.list_tools().await,
        };

        let result = outcome.map_err(AppError::Port)?;
        tracing::info!(operator = %auth.subject, kind = entry.kind, outcome = "ok", "task complete");
        Ok(result)
    }

    /// Route a forge-targeted task kind to the appropriate forge port method.
    async fn dispatch_forge(&self, kind: &str) -> Result<serde_json::Value, fpa_ports::PortError> {
        // `describe` needs a target; everything else is a safe list read until
        // forge's gateway lands richer create/define calls (project.create, etc.).
        if matches!(kind, "forge.table.describe" | "project.inspect") {
            self.forge.describe_table("<unspecified>").await
        } else {
            self.forge.list_tables().await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use fpa_domain::{OperatorId, TaskId};
    use fpa_ports::PortError;
    use std::sync::atomic::{AtomicBool, Ordering};
    use uuid::Uuid;

    // --- fakes: track whether the port was actually called ---
    #[derive(Default)]
    struct FakeForge {
        called: AtomicBool,
    }
    #[async_trait]
    impl ForgeMetadata for FakeForge {
        async fn list_tables(&self) -> Result<serde_json::Value, PortError> {
            self.called.store(true, Ordering::SeqCst);
            Ok(serde_json::json!({"tables": []}))
        }
        async fn describe_table(&self, _: &str) -> Result<serde_json::Value, PortError> {
            self.called.store(true, Ordering::SeqCst);
            Ok(serde_json::json!({}))
        }
    }
    struct FakeFabric;
    #[async_trait]
    impl FabricClient for FakeFabric {
        async fn health(&self) -> Result<(), PortError> {
            Ok(())
        }
    }
    struct FakeGate;
    #[async_trait]
    impl GateAdmin for FakeGate {
        async fn list_routes(&self) -> Result<serde_json::Value, PortError> {
            Ok(serde_json::json!({"routes": []}))
        }
    }
    struct FakeMcp;
    #[async_trait]
    impl McpClient for FakeMcp {
        async fn list_tools(&self) -> Result<serde_json::Value, PortError> {
            Ok(serde_json::json!({"tools": []}))
        }
        async fn call_tool(
            &self,
            _: &str,
            _: serde_json::Value,
        ) -> Result<serde_json::Value, PortError> {
            Ok(serde_json::Value::Null)
        }
    }

    fn runner_with(forge: Arc<FakeForge>) -> TaskRunner {
        TaskRunner::new(
            forge,
            Arc::new(FakeFabric),
            Arc::new(FakeGate),
            Arc::new(FakeMcp),
        )
    }

    fn task(kind: &str) -> AdminTask {
        AdminTask {
            id: TaskId(Uuid::nil()),
            operator: OperatorId(Uuid::nil()),
            kind: kind.to_owned(),
            input: serde_json::Value::Null,
            state: fpa_domain::TaskState::Submitted,
        }
    }

    fn auth(roles: &[&str]) -> AuthContext {
        AuthContext {
            subject: "op-1".into(),
            roles: roles.iter().map(|s| (*s).to_owned()).collect(),
        }
    }

    #[tokio::test]
    async fn unknown_kind_is_rejected() {
        let r = runner_with(Arc::new(FakeForge::default()));
        let err = r
            .run(&task("nope.nope"), &auth(&["admin"]))
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::UnknownTaskKind(_)));
    }

    #[tokio::test]
    async fn dispatches_to_forge_for_viewer() {
        let forge = Arc::new(FakeForge::default());
        let r = runner_with(forge.clone());
        let out = r
            .run(&task("forge.table.list"), &auth(&["viewer"]))
            .await
            .unwrap();
        assert!(
            forge.called.load(Ordering::SeqCst),
            "forge port must be called"
        );
        assert!(out.get("tables").is_some());
    }

    #[tokio::test]
    async fn permission_denied_never_calls_port() {
        let forge = Arc::new(FakeForge::default());
        let r = runner_with(forge.clone());
        // application.deploy requires "admin"; operator only has "viewer".
        let err = r
            .run(&task("application.deploy"), &auth(&["viewer"]))
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Unauthorized(_)));
        assert!(
            !forge.called.load(Ordering::SeqCst),
            "no port call on denial"
        );
    }
}
