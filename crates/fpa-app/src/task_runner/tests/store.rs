//! Store-backed task tests (p7-c001, p7-c002, p8-c001), split from `tests/mod.rs`
//! to keep every test file under the 500-line limit. `use super::*;` resolves the
//! shared fakes, runner builders, and `task`/`auth` helpers declared in `mod.rs`,
//! which resolve in turn against the runner's own `mod.rs` via that module's
//! `use super::*;`.

use super::*;

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
    t2.input = serde_json::json!({ "project_id": pid, "application": app_json(0xA1, "second") });
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
