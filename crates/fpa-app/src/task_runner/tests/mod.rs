//! Unit tests for the A2A task runner (split from `task_runner.rs`, p13-c001).
//!
//! This is the body of the runner's `#[cfg(test)] mod tests` — declared as a
//! directory module by `task_runner/mod.rs`. `use super::*;` resolves against the
//! runner's `mod.rs`. To keep every test file under the 500-line limit, two groups
//! live in submodules that see the shared fakes/helpers here via `super::*`:
//! `store` (p7/p8 store-backed tasks) and `mcp` (MCP dispatch, p13/p14).

use super::*;
use async_trait::async_trait;
use fpa_domain::{OperatorId, TaskId};
use fpa_ports::PortError;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use uuid::Uuid;

mod mcp;
mod store;

// --- fakes: track whether the port was actually called ---
#[derive(Default)]
struct FakeForge {
    called: AtomicBool,
    /// The table name passed to the most recent `describe_table` call, captured so
    /// tests can assert the runner threads the validated `name` through (p13-c002).
    described: Mutex<Option<String>>,
}
#[async_trait]
impl ForgeMetadata for FakeForge {
    async fn list_tables(&self, _bearer: Option<&str>) -> Result<serde_json::Value, PortError> {
        self.called.store(true, Ordering::SeqCst);
        Ok(serde_json::json!({"tables": []}))
    }
    async fn describe_table(
        &self,
        name: &str,
        _bearer: Option<&str>,
    ) -> Result<serde_json::Value, PortError> {
        self.called.store(true, Ordering::SeqCst);
        *self.described.lock().expect("describe capture lock") = Some(name.to_owned());
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
async fn describe_threads_validated_table_name_to_forge() {
    // p13-c002 G1: the runner must forward the validated `name` to the forge port,
    // not a placeholder. Assert the exact argument value reaches describe_table.
    let forge = Arc::new(FakeForge::default());
    let r = runner_with(forge.clone());
    let mut t = task("forge.table.describe");
    t.input = serde_json::json!({ "name": "widgets" });
    r.run(&t, &auth(&["viewer"]))
        .await
        .expect("valid input runs");
    let described = forge.described.lock().expect("capture lock").clone();
    assert_eq!(
        described.as_deref(),
        Some("widgets"),
        "runner must pass the validated table name to the forge port"
    );
}

#[tokio::test]
async fn describe_rejects_empty_table_name() {
    // Schema admits `name: ""`, but an empty/whitespace table must never reach the
    // port — parse_table_name refuses it (p13-c002 G1, task 1.3).
    let forge = Arc::new(FakeForge::default());
    let r = runner_with(forge.clone());
    let mut t = task("forge.table.describe");
    t.input = serde_json::json!({ "name": "   " });
    let err = r.run(&t, &auth(&["viewer"])).await.unwrap_err();
    assert!(matches!(err, AppError::Port(PortError::Downstream(_))));
    assert!(
        !forge.called.load(Ordering::SeqCst),
        "empty table name must not reach the forge port"
    );
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

#[tokio::test]
async fn application_deploy_refusal_names_missing_gate_write_contract() {
    // p14-c002: application.deploy cannot be honestly implemented — the GateAdmin
    // port exposes list_routes only, with no verified flint-gate admin write
    // endpoint. The refusal must SAY SO (never a fake success), so an operator can
    // tell "not built yet" from a real failure. This pins the honest message.
    let gate = Arc::new(FakeGate::default());
    let r = runner_with_gate(Arc::new(FakeForge::default()), gate.clone());
    let err = r
        .run(&task("application.deploy"), &auth(&["admin"]))
        .await
        .unwrap_err();
    let AppError::Port(PortError::Downstream(msg)) = err else {
        panic!("expected a Downstream refusal, got {err:?}");
    };
    assert!(
        msg.contains("no verified flint-gate") && msg.contains("list_routes only"),
        "refusal must name the missing gate admin write contract, got: {msg}"
    );
    assert!(
        !gate.listed.load(Ordering::SeqCst),
        "a refused gate write must not list routes"
    );
}

#[test]
fn every_gate_catalog_kind_is_classified() {
    // p5-c003 G3 (security-review LOW): the gate write-guard uses a manual
    // allowlist, not the catalog. Guard against a future Gate write kind being
    // added to the catalog without being classified — which would silently make
    // it a read (list_routes). Every Gate kind must be a known write or an
    // explicit read.
    const GATE_READ_KINDS: &[&str] = &["gate.route.list"]; // gate read kinds (p13-c003)
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

// ---- p13-c003: gate.route.list read kind (mcp.tool.list/call → tests/mcp.rs) ----

#[tokio::test]
async fn gate_route_list_runs_as_operator_and_lists_routes() {
    // p13-c003: gate.route.list is a Gate read — operator role reaches list_routes,
    // with no write refusal.
    let gate = Arc::new(FakeGate::default());
    let r = runner_with_gate(Arc::new(FakeForge::default()), gate.clone());
    let out = r
        .run(&task("gate.route.list"), &auth(&["operator"]))
        .await
        .expect("operator lists routes");
    assert!(
        gate.listed.load(Ordering::SeqCst),
        "gate.route.list must call list_routes"
    );
    assert!(out.get("routes").is_some(), "returns the routes envelope");
}

#[tokio::test]
async fn gate_route_list_denies_non_operator() {
    // A role below operator (viewer) is rejected by the permission check before any
    // port call — gate topology is an information-disclosure surface.
    let gate = Arc::new(FakeGate::default());
    let r = runner_with_gate(Arc::new(FakeForge::default()), gate.clone());
    let err = r
        .run(&task("gate.route.list"), &auth(&["viewer"]))
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::Unauthorized(_)));
    assert!(
        !gate.listed.load(Ordering::SeqCst),
        "denied read must not call list_routes"
    );
}
