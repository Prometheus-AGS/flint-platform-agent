## 1. ProjectStore port

- [ ] 1.1 Add `ProjectStore` trait to `fpa-ports` (`put(&Project) -> Result<(), PortError>`, `get(&ProjectId) -> Result<Option<Project>, PortError>`, `Send + Sync`, `#[async_trait]` if the durable backend will be async — prefer async for future-proofing).
- [ ] 1.2 Ensure `fpa-ports` depends on `fpa-domain` for `Project` / `ProjectId` (domain-only import; no adapter deps).

## 2. In-memory adapter

- [ ] 2.1 Add `InMemoryProjectStore` (in `fpa-app`, alongside `TaskStore`): `RwLock<HashMap<ProjectId, Project>>`; implement `ProjectStore`.
- [ ] 2.2 `put` clones+inserts by `project.id`; `get` returns a clone or `None`.

## 3. Rewire project.create

- [ ] 3.1 `TaskRunner` holds `Arc<dyn ProjectStore>`; add it to `new(...)` and the struct.
- [ ] 3.2 In dispatch, `project.create` builds a `Project` from validated input (id from input or a generated `ProjectId`; `name` required) and calls `store.put`, returning the stored artifact. Remove the `forge.create_entity("Projects", …)` route.
- [ ] 3.3 Keep the catalog role pre-check; `project.create` still requires its catalogued role.

## 4. Compose at the root

- [ ] 4.1 `fpa-gateway::state::AppState` constructs an `InMemoryProjectStore` (Arc) and passes it into `TaskRunner::new`.

## 5. Verification

- [ ] 5.1 `cargo check/clippy/fmt` green (batched at section boundary).
- [ ] 5.2 (Integration test lands in p5-c004) — unit-level: store put/get round-trips the full aggregate; `project.create` stores + returns, calls no forge write.
