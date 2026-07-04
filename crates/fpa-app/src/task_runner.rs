//! The A2A task runner use-case.
//!
//! Resolves a task `kind` to a fabric operation and executes it through the
//! relevant port(s), enforcing gate-derived permissions and emitting an audit
//! record. Holds ports as trait objects so the composition root can inject any
//! adapter (and tests can inject fakes).

use crate::auth::AuthContext;
use crate::catalog::{self, TargetPort};
use crate::error::AppError;
use fpa_domain::{AdminTask, ApplicationDef, Project, ProjectId};
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
            TargetPort::Store => self.dispatch_store(entry.kind, &input).await,
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
    /// Store-backed kinds (`project.create`, `application.define`) are NOT here —
    /// they route via [`Self::dispatch_store`] (`TargetPort::Store`).
    /// **Unmapped write kinds return a clean `Downstream("write API pending")` —
    /// never a silent read fallback.**
    async fn dispatch_forge(
        &self,
        kind: &str,
        _input: &serde_json::Value,
        bearer: Option<&str>,
    ) -> Result<serde_json::Value, fpa_ports::PortError> {
        match kind {
            // Reads — genuine forge metadata. `project.inspect`/`project.list` are
            // NOT here: they read the agent-owned store (p8-c001, `dispatch_store`).
            "forge.table.describe" => self.forge.describe_table("<unspecified>", bearer).await,
            "forge.table.list" => self.forge.list_tables(bearer).await,
            // Any other forge kind that is write-oriented has no mapping yet.
            other => Err(fpa_ports::PortError::Downstream(format!(
                "write API pending: forge kind '{other}' not implemented"
            ))),
        }
    }

    /// Route an agent-owned aggregate write (`TargetPort::Store`) to its handler.
    /// These mutate the `Project` aggregate in the `ProjectStore` — no external plane.
    async fn dispatch_store(
        &self,
        kind: &str,
        input: &serde_json::Value,
    ) -> Result<serde_json::Value, fpa_ports::PortError> {
        match kind {
            "project.create" => self.create_project(input).await,
            "application.define" => self.define_application(input).await,
            "project.inspect" => self.inspect_project(input).await,
            "project.list" => self.list_projects().await,
            other => Err(fpa_ports::PortError::Downstream(format!(
                "store kind '{other}' not implemented"
            ))),
        }
    }

    /// Read a single project aggregate from the store by `project_id` (p8-c001).
    /// Unknown id → clean error; performs no forge call.
    async fn inspect_project(
        &self,
        input: &serde_json::Value,
    ) -> Result<serde_json::Value, fpa_ports::PortError> {
        let id = parse_project_id(input)?;
        let project = self.projects.get(&id).await?.ok_or_else(|| {
            fpa_ports::PortError::Downstream(format!("unknown project '{}'", id.0))
        })?;
        serde_json::to_value(&project).map_err(|e| fpa_ports::PortError::Decode(e.to_string()))
    }

    /// List all stored projects (p8-c001), deterministically ordered by id
    /// (Base Rule 35). Performs no forge call.
    async fn list_projects(&self) -> Result<serde_json::Value, fpa_ports::PortError> {
        let mut projects = self.projects.list().await?;
        projects.sort_by_key(|p| p.id.0);
        Ok(serde_json::json!({ "projects": projects }))
    }

    /// Persist a new `Project` aggregate to the agent-owned store (p7-c001).
    ///
    /// Builds the aggregate from validated input: `name` required; `id` from
    /// `project_id` or a fresh v4; `schema_version` is **server-owned** (always the
    /// current `SCHEMA_VERSION` — a client value is ignored). The nested collections
    /// (`applications`/`sub_agents`/`schemas`/`realtime`/`entity_meta`) are
    /// serde-deserialized into their typed forms; a malformed nested item is rejected
    /// BEFORE any store write. Performs no forge write — the Project has no forge table.
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
        // Start from an empty aggregate (server owns id + schema_version), then map
        // any nested input onto it via serde — deserialization IS the validation.
        let mut project = Project::new(id, name);
        deser_field(input, "applications", &mut project.applications)?;
        deser_field(input, "sub_agents", &mut project.sub_agents)?;
        deser_field(input, "schemas", &mut project.schemas)?;
        deser_field(input, "entity_meta", &mut project.entity_meta)?;
        if let Some(rt) = input.get("realtime") {
            project.realtime = serde_json::from_value(rt.clone()).map_err(|e| {
                fpa_ports::PortError::Downstream(format!("invalid 'realtime': {e}"))
            })?;
        }
        self.projects.put(&project).await?;
        serde_json::to_value(&project).map_err(|e| fpa_ports::PortError::Decode(e.to_string()))
    }

    /// Define (upsert) an application within an existing project (p7-c002).
    ///
    /// Loads the project by `project_id` from the store, upserts the supplied
    /// `ApplicationDef` by its id (replace same-id, else append — immutably), and
    /// persists the mutated aggregate. An unknown `project_id` is rejected; no
    /// project is created implicitly.
    async fn define_application(
        &self,
        input: &serde_json::Value,
    ) -> Result<serde_json::Value, fpa_ports::PortError> {
        let project_id = parse_project_id(input)?;
        let app: ApplicationDef = input
            .get("application")
            .ok_or_else(|| {
                fpa_ports::PortError::Downstream(
                    "application.define requires 'application'".to_owned(),
                )
            })
            .and_then(|v| {
                serde_json::from_value(v.clone()).map_err(|e| {
                    fpa_ports::PortError::Downstream(format!("invalid 'application': {e}"))
                })
            })?;

        let mut project = self.projects.get(&project_id).await?.ok_or_else(|| {
            fpa_ports::PortError::Downstream(format!("unknown project '{}'", project_id.0))
        })?;

        // Immutable upsert by application id: keep all others, drop the same id, append.
        let mut apps: Vec<ApplicationDef> = project
            .applications
            .into_iter()
            .filter(|a| a.id != app.id)
            .collect();
        apps.push(app);
        project.applications = apps;

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

/// Parse a **required** `project_id` (a UUID string) from task input into a
/// [`ProjectId`]. Missing/non-string → `Downstream("… requires 'project_id'")`;
/// unparseable → `Downstream("invalid project_id: …")`. Shared by the store-backed
/// kinds that operate on an existing project (`application.define`,
/// `project.inspect`). `project.create` treats `project_id` as *optional* and does
/// not use this.
fn parse_project_id(input: &serde_json::Value) -> Result<ProjectId, fpa_ports::PortError> {
    input
        .get("project_id")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| fpa_ports::PortError::Downstream("requires 'project_id'".to_owned()))
        .and_then(|s| {
            Uuid::parse_str(s)
                .map(ProjectId)
                .map_err(|e| fpa_ports::PortError::Downstream(format!("invalid project_id: {e}")))
        })
}

/// Deserialize an optional nested array field from `input` into `target`, if present.
///
/// Absent → leave `target` untouched (its `#[serde(default)]` empty). Present but
/// malformed → a `Downstream` error naming the field (rejected before any store
/// write). This is where "serde IS the validation" is enforced for the nested
/// `Project` collections (p7-c001).
fn deser_field<T: serde::de::DeserializeOwned>(
    input: &serde_json::Value,
    field: &str,
    target: &mut T,
) -> Result<(), fpa_ports::PortError> {
    if let Some(v) = input.get(field) {
        *target = serde_json::from_value(v.clone())
            .map_err(|e| fpa_ports::PortError::Downstream(format!("invalid '{field}': {e}")))?;
    }
    Ok(())
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
        async fn subscribe(
            &self,
            _channel: uuid::Uuid,
            _bearer: Option<&str>,
        ) -> Result<fpa_ports::EventStream, PortError> {
            let empty = futures::stream::empty::<Result<fpa_ports::EventEnvelope, PortError>>();
            Ok(Box::pin(empty))
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

    /// A runner sharing the given store, so a test can assert on stored state.
    fn runner_with_store(store: Arc<crate::project_store::InMemoryProjectStore>) -> TaskRunner {
        TaskRunner::new(
            Arc::new(FakeForge::default()),
            Arc::new(FakeFabric),
            Arc::new(FakeGate::default()),
            Arc::new(FakeMcp),
            store,
        )
    }

    /// A runner sharing both the forge fake (to assert no-forge-call on store reads)
    /// and the store.
    fn runner_with_gate_forge_store(
        forge: Arc<FakeForge>,
        store: Arc<crate::project_store::InMemoryProjectStore>,
    ) -> TaskRunner {
        TaskRunner::new(
            forge,
            Arc::new(FakeFabric),
            Arc::new(FakeGate::default()),
            Arc::new(FakeMcp),
            store,
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

    // ---- p7-c001: richer project.create ----

    fn app_json(id: u128, name: &str) -> serde_json::Value {
        serde_json::json!({
            "id": Uuid::from_u128(id),
            "name": name,
            "components": [],
            "modules": [],
            "plugins": []
        })
    }

    #[tokio::test]
    async fn project_create_stores_full_nested_aggregate() {
        let store = Arc::new(crate::project_store::InMemoryProjectStore::new());
        let r = runner_with_store(store.clone());
        let pid = Uuid::from_u128(0x7001);
        let mut t = task("project.create");
        t.input = serde_json::json!({
            "name": "rich", "project_id": pid,
            "applications": [ app_json(0xA1, "app-one") ]
        });
        let out = r.run(&t, &auth(&["operator"])).await.expect("create");
        assert_eq!(out["applications"][0]["name"], "app-one");
        // Persisted with the nested application.
        let stored = store.get(&ProjectId(pid)).await.unwrap().expect("stored");
        assert_eq!(stored.applications.len(), 1);
        assert_eq!(stored.applications[0].name, "app-one");
    }

    #[tokio::test]
    async fn project_create_rejects_malformed_nested_and_stores_nothing() {
        let store = Arc::new(crate::project_store::InMemoryProjectStore::new());
        let r = runner_with_store(store.clone());
        let pid = Uuid::from_u128(0x7002);
        let mut t = task("project.create");
        // `applications` array with a wrong-typed item (missing/!string name).
        t.input = serde_json::json!({
            "name": "bad", "project_id": pid,
            "applications": [ { "id": Uuid::from_u128(1), "name": 123 } ]
        });
        let err = r.run(&t, &auth(&["operator"])).await.unwrap_err();
        assert!(matches!(err, AppError::Port(PortError::Downstream(_))));
        assert!(
            store.get(&ProjectId(pid)).await.unwrap().is_none(),
            "no project stored on malformed nested input"
        );
    }

    #[tokio::test]
    async fn project_create_ignores_client_schema_version() {
        let store = Arc::new(crate::project_store::InMemoryProjectStore::new());
        let r = runner_with_store(store.clone());
        let pid = Uuid::from_u128(0x7003);
        let mut t = task("project.create");
        t.input = serde_json::json!({ "name": "v", "project_id": pid, "schema_version": 999 });
        r.run(&t, &auth(&["operator"])).await.expect("create");
        let stored = store.get(&ProjectId(pid)).await.unwrap().expect("stored");
        assert_eq!(
            stored.schema_version,
            fpa_domain::SCHEMA_VERSION,
            "server owns schema_version"
        );
    }

    #[tokio::test]
    async fn project_create_routes_via_store_not_other_ports() {
        // Store arm: no forge/gate/mcp call for project.create.
        let forge = Arc::new(FakeForge::default());
        let gate = Arc::new(FakeGate::default());
        let r = runner_with_gate(forge.clone(), gate.clone());
        let mut t = task("project.create");
        t.input = serde_json::json!({ "name": "routed" });
        r.run(&t, &auth(&["operator"])).await.expect("create");
        assert!(!forge.called.load(Ordering::SeqCst), "no forge call");
        assert!(!gate.listed.load(Ordering::SeqCst), "no gate call");
    }

    // ---- p7-c002: application.define ----

    async fn seed_project(r: &TaskRunner, pid: Uuid) {
        // The runner already holds the store; callers assert on it directly.
        let mut t = task("project.create");
        t.input = serde_json::json!({ "name": "host", "project_id": pid });
        r.run(&t, &auth(&["operator"])).await.expect("seed");
    }

    #[tokio::test]
    async fn application_define_upserts_into_existing_project() {
        let store = Arc::new(crate::project_store::InMemoryProjectStore::new());
        let r = runner_with_store(store.clone());
        let pid = Uuid::from_u128(0x7010);
        seed_project(&r, pid).await;

        // Define app A1.
        let mut t = task("application.define");
        t.input = serde_json::json!({ "project_id": pid, "application": app_json(0xA1, "first") });
        r.run(&t, &auth(&["operator"])).await.expect("define");
        // Re-define same id with a new name → upsert (single entry, second wins).
        let mut t2 = task("application.define");
        t2.input =
            serde_json::json!({ "project_id": pid, "application": app_json(0xA1, "second") });
        r.run(&t2, &auth(&["operator"])).await.expect("redefine");

        let stored = store.get(&ProjectId(pid)).await.unwrap().expect("stored");
        assert_eq!(stored.applications.len(), 1, "upsert, not duplicate");
        assert_eq!(stored.applications[0].name, "second");
    }

    #[tokio::test]
    async fn application_define_rejects_unknown_project() {
        let store = Arc::new(crate::project_store::InMemoryProjectStore::new());
        let r = runner_with_store(store.clone());
        let pid = Uuid::from_u128(0x7011);
        let mut t = task("application.define");
        t.input = serde_json::json!({ "project_id": pid, "application": app_json(0xA1, "x") });
        let err = r.run(&t, &auth(&["operator"])).await.unwrap_err();
        assert!(matches!(err, AppError::Port(PortError::Downstream(_))));
        assert!(
            store.get(&ProjectId(pid)).await.unwrap().is_none(),
            "unknown project not created implicitly"
        );
    }

    // ---- p8-c001: store-backed reads ----

    #[tokio::test]
    async fn project_inspect_reads_from_store_without_forge() {
        let store = Arc::new(crate::project_store::InMemoryProjectStore::new());
        let forge = Arc::new(FakeForge::default());
        let r = runner_with_gate_forge_store(forge.clone(), store.clone());
        let pid = Uuid::from_u128(0x8001);
        seed_project(&r, pid).await;

        let mut t = task("project.inspect");
        t.input = serde_json::json!({ "project_id": pid });
        let out = r.run(&t, &auth(&["viewer"])).await.expect("inspect");
        assert_eq!(out["name"], "host");
        assert!(
            !forge.called.load(Ordering::SeqCst),
            "project.inspect must not call forge"
        );
    }

    #[tokio::test]
    async fn project_inspect_unknown_is_error() {
        let r = runner_with_store(Arc::new(crate::project_store::InMemoryProjectStore::new()));
        let mut t = task("project.inspect");
        t.input = serde_json::json!({ "project_id": Uuid::from_u128(0x8002) });
        let err = r.run(&t, &auth(&["viewer"])).await.unwrap_err();
        assert!(matches!(err, AppError::Port(PortError::Downstream(_))));
    }

    #[tokio::test]
    async fn project_list_returns_all_sorted_without_forge() {
        let store = Arc::new(crate::project_store::InMemoryProjectStore::new());
        let forge = Arc::new(FakeForge::default());
        let r = runner_with_gate_forge_store(forge.clone(), store.clone());
        // Seed two projects with ids that would sort b-then-a if unsorted.
        seed_project(&r, Uuid::from_u128(0x8020)).await;
        seed_project(&r, Uuid::from_u128(0x8010)).await;

        let out = r
            .run(&task("project.list"), &auth(&["viewer"]))
            .await
            .expect("list");
        let projects = out["projects"].as_array().expect("array");
        assert_eq!(projects.len(), 2);
        // Deterministic order by id (Base Rule 35).
        assert_eq!(
            projects[0]["id"],
            serde_json::json!(Uuid::from_u128(0x8010))
        );
        assert_eq!(
            projects[1]["id"],
            serde_json::json!(Uuid::from_u128(0x8020))
        );
        assert!(
            !forge.called.load(Ordering::SeqCst),
            "project.list must not call forge"
        );
    }

    #[tokio::test]
    async fn project_list_empty_store_is_empty_list() {
        let r = runner_with_store(Arc::new(crate::project_store::InMemoryProjectStore::new()));
        let out = r
            .run(&task("project.list"), &auth(&["viewer"]))
            .await
            .expect("list");
        assert_eq!(out["projects"].as_array().expect("array").len(), 0);
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

    #[test]
    fn every_store_catalog_kind_is_dispatched() {
        // p7 (rust-review LOW): guard against a future TargetPort::Store kind being
        // catalogued without a `dispatch_store` arm — it would hit the clean catch-all
        // ("store kind not implemented") rather than its intended handler.
        const STORE_KINDS: &[&str] = &[
            "project.create",
            "application.define",
            "project.inspect",
            "project.list",
        ];
        for entry in catalog::CATALOG
            .iter()
            .filter(|e| e.target == TargetPort::Store)
        {
            assert!(
                STORE_KINDS.contains(&entry.kind),
                "Store catalog kind '{}' has no dispatch_store arm — add it or update STORE_KINDS",
                entry.kind
            );
        }
    }
}
