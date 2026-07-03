//! The A2A task runner use-case.
//!
//! Resolves a task `kind` to a fabric operation and executes it through the
//! relevant port(s), enforcing gate-derived permissions and emitting an audit
//! record. Holds ports as trait objects so the composition root can inject any
//! adapter (and tests can inject fakes).

use crate::auth::AuthContext;
use crate::catalog::{self, TargetPort};
use crate::error::AppError;
use fpa_domain::{AdminTask, Project, ProjectId};
use fpa_ports::{FabricClient, ForgeMetadata, GateAdmin, McpClient, ProjectStore};
use std::sync::Arc;
use uuid::Uuid;

/// Executes administrative tasks against the fabric through injected ports.
pub struct TaskRunner {
    pub forge: Arc<dyn ForgeMetadata>,
    pub fabric: Arc<dyn FabricClient>,
    pub gate: Arc<dyn GateAdmin>,
    pub mcp: Arc<dyn McpClient>,
    /// Agent-owned persistence for the `Project` aggregate (p5-c001). The Project
    /// has no forge table; `project.create` writes here, not to forge.
    pub projects: Arc<dyn ProjectStore>,
}

impl TaskRunner {
    /// Construct a runner from the four plane ports plus the project store.
    #[must_use]
    pub fn new(
        forge: Arc<dyn ForgeMetadata>,
        fabric: Arc<dyn FabricClient>,
        gate: Arc<dyn GateAdmin>,
        mcp: Arc<dyn McpClient>,
        projects: Arc<dyn ProjectStore>,
    ) -> Self {
        Self {
            forge,
            fabric,
            gate,
            mcp,
            projects,
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

        // Validate input against the kind's JSON Schema before any port call.
        // Treat null/absent input as an empty object so no-argument calls pass an
        // object schema while required fields are still enforced when expected.
        let schema = entry.input_schema();
        let input = if task.input.is_null() {
            serde_json::json!({})
        } else {
            task.input.clone()
        };
        if let Err(err) = jsonschema::validate(&schema, &input) {
            return Err(AppError::InvalidInput(format!(
                "input for '{}' failed schema: {err}",
                entry.kind
            )));
        }

        // Audit the allow decision, recording identity provenance (p5-c003 G6):
        // whether the identity was signature-verified (direct token) or
        // gate-trusted (injected headers). Never log the token/claims.
        tracing::info!(
            operator = %auth.subject,
            kind = entry.kind,
            decision = "allowed",
            signature_verified = auth.signature_verified,
            "task dispatch"
        );

        // Dispatch to the mapped plane. Concrete per-kind argument mapping lands
        // as the ports gain real implementations; here we route by target so the
        // contract + permission gate are exercised end-to-end.
        let outcome = match entry.target {
            TargetPort::Forge => {
                self.dispatch_forge(entry.kind, &input, auth.bearer.as_deref())
                    .await
            }
            TargetPort::Fabric => self
                .fabric
                .health()
                .await
                .map(|()| serde_json::json!({"ok": true})),
            TargetPort::Gate => self.dispatch_gate(entry.kind).await,
            TargetPort::Mcp => self.mcp.list_tools().await,
        };

        let result = outcome.map_err(AppError::Port)?;
        tracing::info!(
            operator = %auth.subject,
            kind = entry.kind,
            outcome = "ok",
            signature_verified = auth.signature_verified,
            "task complete"
        );
        Ok(result)
    }

    /// Route a forge-targeted task kind to the appropriate forge port method,
    /// forwarding the operator's bearer for RLS. Reads map to metadata/queries.
    /// `project.create` does **not** appear here — the Project aggregate persists
    /// in the agent-owned store (see [`Self::create_project`]), not forge (p5-c001).
    /// **Unmapped write kinds return a clean `Downstream("write API pending")` —
    /// never a silent read fallback.**
    async fn dispatch_forge(
        &self,
        kind: &str,
        input: &serde_json::Value,
        bearer: Option<&str>,
    ) -> Result<serde_json::Value, fpa_ports::PortError> {
        match kind {
            // Reads
            "forge.table.describe" | "project.inspect" => {
                self.forge.describe_table("<unspecified>", bearer).await
            }
            "forge.table.list" | "project.list" => self.forge.list_tables(bearer).await,
            // `project.create` is store-backed and handled before dispatch_forge is
            // reached; `application.define` writes a forge-backed application row.
            "project.create" => self.create_project(input).await,
            // Any other forge kind that is write-oriented has no mapping yet.
            other => Err(fpa_ports::PortError::Downstream(format!(
                "write API pending: forge kind '{other}' not implemented"
            ))),
        }
    }

    /// Persist a new `Project` aggregate to the agent-owned store (p5-c001).
    ///
    /// Builds the aggregate from validated input (`name` required, optional
    /// `project_id`), stores it, and returns the stored artifact. Performs no
    /// forge write — the Project has no forge table.
    async fn create_project(
        &self,
        input: &serde_json::Value,
    ) -> Result<serde_json::Value, fpa_ports::PortError> {
        // Input is schema-validated upstream: `name` is required.
        let name = input
            .get("name")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                fpa_ports::PortError::Downstream("project.create requires 'name'".to_owned())
            })?;
        // Deterministic when the caller supplies `project_id`; else a fresh v4.
        let id = match input.get("project_id").and_then(serde_json::Value::as_str) {
            Some(s) => ProjectId(Uuid::parse_str(s).map_err(|e| {
                fpa_ports::PortError::Downstream(format!("invalid project_id: {e}"))
            })?),
            None => ProjectId(Uuid::new_v4()),
        };
        let project = Project::new(id, name);
        self.projects.put(&project).await?;
        serde_json::to_value(&project).map_err(|e| fpa_ports::PortError::Decode(e.to_string()))
    }

    /// Dispatch a gate-targeted task kind (p5-c003 G3). Read kinds call
    /// `list_routes`; **write kinds refuse cleanly** rather than silently listing
    /// routes. Gate route-writes are not implemented this phase.
    async fn dispatch_gate(&self, kind: &str) -> Result<serde_json::Value, fpa_ports::PortError> {
        if is_gate_write_kind(kind) {
            return Err(fpa_ports::PortError::Downstream(format!(
                "gate route-write not implemented: '{kind}'"
            )));
        }
        self.gate.list_routes().await
    }
}

/// Whether a gate-targeted task kind performs a write (route/auth mutation).
///
/// Write kinds must refuse (see [`TaskRunner::dispatch_gate`]) until gate
/// route-writes are implemented. Read kinds fall through to `list_routes`.
fn is_gate_write_kind(kind: &str) -> bool {
    matches!(kind, "application.deploy")
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
        async fn list_tables(&self, _bearer: Option<&str>) -> Result<serde_json::Value, PortError> {
            self.called.store(true, Ordering::SeqCst);
            Ok(serde_json::json!({"tables": []}))
        }
        async fn describe_table(
            &self,
            _: &str,
            _bearer: Option<&str>,
        ) -> Result<serde_json::Value, PortError> {
            self.called.store(true, Ordering::SeqCst);
            Ok(serde_json::json!({}))
        }
        async fn create_entity(
            &self,
            schema: &str,
            table: &str,
            _object: serde_json::Value,
            _bearer: Option<&str>,
        ) -> Result<serde_json::Value, PortError> {
            self.called.store(true, Ordering::SeqCst);
            Ok(serde_json::json!({ "created_in": format!("{schema}.{table}") }))
        }
    }
    struct FakeFabric;
    #[async_trait]
    impl FabricClient for FakeFabric {
        async fn health(&self) -> Result<(), PortError> {
            Ok(())
        }
    }
    #[derive(Default)]
    struct FakeGate {
        listed: AtomicBool,
    }
    #[async_trait]
    impl GateAdmin for FakeGate {
        async fn list_routes(&self) -> Result<serde_json::Value, PortError> {
            self.listed.store(true, Ordering::SeqCst);
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
        runner_with_gate(forge, Arc::new(FakeGate::default()))
    }

    fn runner_with_gate(forge: Arc<FakeForge>, gate: Arc<FakeGate>) -> TaskRunner {
        TaskRunner::new(
            forge,
            Arc::new(FakeFabric),
            gate,
            Arc::new(FakeMcp),
            Arc::new(crate::project_store::InMemoryProjectStore::new()),
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
            bearer: Some("test-bearer".into()),
            signature_verified: true,
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
    async fn invalid_input_never_calls_port() {
        let forge = Arc::new(FakeForge::default());
        let r = runner_with(forge.clone());
        // forge.table.describe requires `name`; provide an object without it.
        let mut t = task("forge.table.describe");
        t.input = serde_json::json!({ "wrong": 1 });
        let err = r.run(&t, &auth(&["viewer"])).await.unwrap_err();
        assert!(matches!(err, AppError::InvalidInput(_)));
        assert!(
            !forge.called.load(Ordering::SeqCst),
            "no port call on invalid input"
        );
    }

    #[tokio::test]
    async fn valid_required_input_passes() {
        let forge = Arc::new(FakeForge::default());
        let r = runner_with(forge.clone());
        let mut t = task("forge.table.describe");
        t.input = serde_json::json!({ "name": "customers" });
        r.run(&t, &auth(&["viewer"]))
            .await
            .expect("valid input runs");
        assert!(forge.called.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn project_create_stores_and_returns_without_forge_write() {
        // p5-c001: project.create persists to the agent-owned store, not forge.
        let forge = Arc::new(FakeForge::default());
        let r = runner_with(forge.clone());
        let mut t = task("project.create");
        t.input = serde_json::json!({ "name": "alpha" });
        let out = r.run(&t, &auth(&["operator"])).await.unwrap();
        assert!(
            !forge.called.load(Ordering::SeqCst),
            "project.create must NOT call a forge write"
        );
        assert_eq!(out["name"], "alpha");
        assert_eq!(out["schema_version"], fpa_domain::SCHEMA_VERSION);
    }

    #[tokio::test]
    async fn project_create_requires_name() {
        // Schema requires `name`; absence fails validation before any store write.
        let r = runner_with(Arc::new(FakeForge::default()));
        let err = r
            .run(&task("project.create"), &auth(&["operator"]))
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::InvalidInput(_)));
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

    #[tokio::test]
    async fn gate_write_kind_refuses_without_listing_routes() {
        // p5-c003 G3: application.deploy (a Gate write) must refuse cleanly and
        // MUST NOT fall through to list_routes.
        let gate = Arc::new(FakeGate::default());
        let r = runner_with_gate(Arc::new(FakeForge::default()), gate.clone());
        let err = r
            .run(&task("application.deploy"), &auth(&["admin"]))
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Port(PortError::Downstream(_))));
        assert!(
            !gate.listed.load(Ordering::SeqCst),
            "a gate write must not list routes"
        );
    }

    #[test]
    fn every_gate_catalog_kind_is_classified() {
        // p5-c003 G3 (security-review LOW): the gate write-guard uses a manual
        // allowlist, not the catalog. Guard against a future Gate write kind being
        // added to the catalog without being classified — which would silently make
        // it a read (list_routes). Every Gate kind must be a known write or an
        // explicit read.
        const GATE_READ_KINDS: &[&str] = &[]; // add gate read kinds here as they land
        for entry in catalog::CATALOG
            .iter()
            .filter(|e| e.target == TargetPort::Gate)
        {
            assert!(
                is_gate_write_kind(entry.kind) || GATE_READ_KINDS.contains(&entry.kind),
                "Gate catalog kind '{}' is unclassified — add it to is_gate_write_kind or GATE_READ_KINDS",
                entry.kind
            );
        }
    }
}
