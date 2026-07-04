## 1. TargetPort::Store routing

- [x] 1.1 Add `Store` variant to `TargetPort` (`catalog.rs`); keep `#[non_exhaustive]`.
- [x] 1.2 Retarget `project.create` catalog entry to `TargetPort::Store`.
- [x] 1.3 In `task_runner::run`, add a `TargetPort::Store => self.dispatch_store(entry.kind, &input).await` arm; `dispatch_store` matches store-backed kinds.

## 2. Richer create_project

- [x] 2.1 Update the `project.create` catalog schema: `required: ["name"]`; allow optional `project_id` (string) + `applications`/`sub_agents`/`schemas`/`realtime`/`entity_meta` (arrays/object) — light guard only.
- [x] 2.2 In `create_project`, build the `Project`: id from `project_id`/fresh v4; `schema_version` = `SCHEMA_VERSION` (ignore any client value); deserialize the nested fields into the typed collections (e.g. `serde_json::from_value::<Vec<ApplicationDef>>(...)`), mapping serde errors → `AppError::InvalidInput`/`PortError::Downstream` BEFORE `put`.
- [x] 2.3 Persist via `ProjectStore.put`; return the stored aggregate.

## 3. Verification

- [x] 3.1 `cargo check/clippy/fmt` green (batched).
- [x] 3.2 Unit: full nested payload → stored Project contains the nested items; round-trips.
- [x] 3.3 Unit: malformed nested item → InvalidInput/Downstream, no store write (assert store empty).
- [x] 3.4 Unit: client-supplied `schema_version` is ignored (stored = SCHEMA_VERSION).
- [x] 3.5 Unit: `project.create` routes via the Store arm (no forge/gate/mcp call) — reuse the fakes' called-flags.
