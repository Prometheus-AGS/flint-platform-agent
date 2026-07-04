## 1. Core Project aggregate

- [x] 1.1 `fpa-domain/src/project/mod.rs`: `Project { id, schema_version, name, applications, sub_agents, schemas, realtime, entity_meta }` with newtype `ProjectId`.
- [x] 1.2 `ApplicationDef` (components, modules, WASM plugin refs) + `ApplicationId`.
- [x] 1.3 `SubAgentDef` + `SubAgentId`; `ComponentRef` (references a forge A2UI registry entry by id/version — does NOT embed component source).
- [x] 1.4 `SchemaDef` (table/schema defs carrying A2UI generation hints as `serde_json::Value`) + `RealtimeParams`; `EntityMetaRef` (opaque identifier into prometheus-entity-management — no Rust dep).
- [x] 1.5 All public enums `#[non_exhaustive]`; all IDs `#[repr(transparent)]` newtypes; serde derive throughout.

## 2. Versioning + schema

- [x] 2.1 Add `schema_version` constant and a `SCHEMA_VERSION` newtype; document the migration policy (additive-by-default).
- [x] 2.2 Generate/author a JSON Schema for `Project` (e.g. via `schemars` — verify version at impl time) under `crates/fpa-domain/schema/project.schema.json`.
- [x] 2.3 Decide schemars-derive vs hand-authored schema (note tradeoff in the change); keep the schema checked in and versioned.

## 3. A2UI registry conformance

- [x] 3.1 Document in the module that `ComponentRef` aligns to forge `RFC-FORGE-A2UI-001` (registry id + version), NOT an agent-local vocabulary.
- [x] 3.2 Provide a placeholder `RegistryComponentId` newtype to be resolved against the forge registry when it ships.

## 4. Verification

- [x] 4.1 `cargo check/clippy/fmt` green; files <500 lines.
- [x] 4.2 Round-trip test: `Project` serializes to JSON and deserializes back identically (proptest or table-driven).
- [x] 4.3 Test: a sample `Project` JSON validates against `project.schema.json`.
- [x] 4.4 Test: unknown future enum variants deserialize without panic (forward-compat).
