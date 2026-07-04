## Why

The assessment identifies the **"Project" artifact** as the hub every other requirement plugs into (KB, OpenDesign export, app generation). It has no representation yet. Defining it first — typed, versioned, JSON-Schema'd (Base Rule 39) — unblocks the A2A task catalog and all later phases.

## What Changes

- Add a `Project` aggregate to `fpa-domain`: a collection of A2UI component references, sub-agent definitions, application definitions (which contain A2UI component collections, modules, WASM plugin refs), database/schema definitions (carrying A2UI generation hints), realtime parameters, and entity-management metadata.
- Model child types: `ApplicationDef`, `SubAgentDef`, `ComponentRef`, `SchemaDef`, `RealtimeParams`, `EntityMetaRef` — all `#[non_exhaustive]`, serde, newtype IDs.
- A2UI component references **conform to forge's `RFC-FORGE-A2UI-001` registry** (they reference registry entries; this agent does not define the components — see CLAUDE.md A2UI ownership).
- Emit a JSON Schema for the `Project` artifact (versioned `schema_version`) so generated apps and external tools can validate it.
- Entity metadata references the TS `prometheus-entity-management` model by identifier only (no Rust dependency on that workspace).

## Capabilities

### New Capabilities
- `project-artifact`: The versioned `Project` domain aggregate and its child types, with a published JSON Schema and serde round-trip guarantees.

### Modified Capabilities

## Impact

- `fpa-domain` (new `project` module tree, kept under 500 lines/file).
- New `serde`-only types; no infra deps (Layer 0 stays pure).
- Establishes the contract consumed by `p1-c003` (task catalog) and later KB/export/generation phases.
