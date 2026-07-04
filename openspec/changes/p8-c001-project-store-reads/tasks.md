## 1. ProjectStore::list port method

- [ ] 1.1 Add `async fn list(&self) -> Result<Vec<Project>, PortError>` to the `ProjectStore` trait (`fpa-ports`).
- [ ] 1.2 `InMemoryProjectStore::list`: `Ok(self.projects.read().await.values().cloned().collect())`.
- [ ] 1.3 `PgProjectStore::list`: `SELECT body FROM fpa_projects` via `client.query`; map each row's `body` through `serde_json::from_value` → `Vec<Project>`; errors → `Downstream`/`Decode`.

## 2. Retarget the reads to the store

- [ ] 2.1 Catalog: retarget `project.inspect` and `project.list` to `TargetPort::Store`.
- [ ] 2.2 `dispatch_forge`: remove `project.inspect` / `project.list` from the shared arms (leave `forge.table.describe` / `forge.table.list`).
- [ ] 2.3 `dispatch_store`: add `"project.inspect"` → parse `project_id`, `store.get`, `None` → `Downstream("unknown project '<id>'")`, else return the aggregate.
- [ ] 2.4 `dispatch_store`: add `"project.list"` → `store.list()`, sort by id (deterministic), return `{"projects":[…]}`.
- [ ] 2.5 Update the `every_store_catalog_kind_is_dispatched` test's `STORE_KINDS` to include `project.inspect` + `project.list`.

## 3. Verification

- [ ] 3.1 `cargo check/clippy/fmt` green (batched).
- [ ] 3.2 Unit: `project.inspect` returns a stored project; unknown id → Downstream; no forge call (fakes' called-flag).
- [ ] 3.3 Unit: `project.list` returns all stored projects (create 2, list → 2), deterministic order; empty store → empty list; no forge call.
- [ ] 3.4 Add a `list` assertion inside the existing `#[ignore]`d `fpa-store-pg` container test (Docker-gated; not run this session).
