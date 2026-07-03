## Why

`fpa-forge`'s REST insert path is **wrong** and would 404 at runtime. It POSTs to
`{base}/rest/<table>` (single segment, `/rest` prefix), but forge's real REST insert
route is `POST /{schema}/{table}` — **two** segments (schema first), merged at the
gateway **root with no `/rest` prefix**. Evidence, source-verified this phase:
- `flint-forge/crates/fdb-reflection/src/compilers/rest/mod.rs:258` →
  `format!("/{schema}/{table}")` is the generated route path.
- `fdb-gateway/src/main.rs:182-192` → the reflection router is `.merge`d onto the
  root app (no `.nest("/rest", …)`).
- `mutations.rs:78` → handler signature is `Path((schema, table))`.

This is a correctness fix decoupled from `project.create` (which now uses the
agent-owned `ProjectStore`, per p5-c001) — but the forge REST write path must be
right for any real forge-table write and for the integration proof's forge mock.

## What Changes

- Change `fpa-forge::rest_insert` to build `{base}/{schema}/{table}` (schema +
  table, no `/rest` prefix). Take **schema** and **table** as parameters.
- Replace the `rest_prefix` config with a **schema** notion: `FPA_FORGE_REST_PREFIX`
  becomes an optional path base (default empty, since forge merges at root);
  `create_entity` gains a `schema` argument (or a `{schema, table}` pair).
- Update `create_entity`'s signature/callers accordingly. Since `project.create` no
  longer calls forge writes (p5-c001), the remaining forge-write callers pass an
  explicit `{schema, table}`.
- Update the wiremock tests to assert `POST /{schema}/{table}`.

## Capabilities

### New Capabilities
- `forge-rest-path`: Correct forge REST insert addressing (`POST /{schema}/{table}`, no `/rest` prefix), schema-qualified and config-safe.

### Modified Capabilities

## Impact

- `fpa-forge` (path construction + `create_entity`/`rest_insert` signature + tests),
  `fpa-gateway::config` (`FPA_FORGE_REST_PREFIX` semantics; default empty),
  `fpa-app` any forge-write callers pass `{schema, table}`.
- No new dependencies.

## Open Questions
- **RESOLVED:** real path is `POST /{schema}/{table}` at the gateway root (source-verified). The default forge schema for administrative entities is a config concern; no such table exists yet, so no default schema is hardcoded.
