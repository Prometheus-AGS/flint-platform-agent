# Execution — a2a-task-catalog-writes (p14)

**Backend:** OpenSpec (`openspec/` present; changes authored under `openspec/changes/`)
**Driver:** `/kbd-apply` walks each change's `tasks.md`, firing `task:before`/`task:after`.
**Changes:** 2, ordered c001 → c002 (diff-hygiene — both edit `task_runner/mod.rs`).
**Test budget:** ONE `cargo test -p fpa-app` covers both changes (≤3 for the phase).

## Backend selection

OpenSpec is the resolved backend: `openspec/` exists at the repo root and both
changes are already authored as `openspec/changes/p14-c001-*` and
`openspec/changes/p14-c002-*` (validated: `openspec validate` reports both valid).
Spec-backed traceability is required for a catalog change that alters the A2A
surface contract, so `native-tool`/`manual` are not appropriate.

## Dispatch contract

### p14-c001-mcp-tool-call  *(real deliverable)*

Files: `crates/fpa-app/src/catalog.rs`, `crates/fpa-app/src/task_runner/mod.rs`,
`crates/fpa-app/src/task_runner/tests/` (split — see below).

1. **Catalog** — add `SCHEMA_MCP_TOOL_CALL`
   (`{"type":"object","required":["name","arguments"],"properties":{"name":{"type":"string"},"arguments":{"type":"object"}}}`)
   and a `mcp.tool.call` entry → `TargetPort::Mcp`, `required_role: "operator"`
   (invoke floor, not viewer; [[topology-reads-default-role-up]] is about *reads* —
   an invoke floors higher regardless).
2. **Runner** — replace the flat `TargetPort::Mcp => self.mcp.list_tools().await`
   arm with `TargetPort::Mcp => self.dispatch_mcp(entry.kind, &input).await`.
   `dispatch_mcp`: `mcp.tool.list` → `list_tools`; `mcp.tool.call` → parse
   `{name, arguments}` via a `parse_tool_call` helper → `call_tool`; any other Mcp
   kind → clean `PortError::Downstream("mcp kind '…' not implemented")` — **never a
   `list_tools` fallback**.
3. **Guard** — extend `every_mcp_catalog_kind_is_dispatched`'s `MCP_KINDS` to
   `&["mcp.tool.list", "mcp.tool.call"]`.
4. **Audit/idempotency** — the existing runner audit logs `operator`, `kind`,
   `decision`/`outcome`, `signature_verified` only; `arguments` are NEVER logged
   (asserted by a dedicated test capturing `tracing`). Document `mcp.tool.call` as
   **non-idempotent** in a doc-comment (a retried invoke may act twice).

### p14-c002-application-deploy-refused-write  *(honest-refusal message + docs)*

Files: `crates/fpa-app/src/task_runner/mod.rs` (refusal message string only),
`task_runner/tests/` (one regression test), the c002 spec, and a memory.

- `dispatch_gate` still refuses `application.deploy` — tighten the message to name
  the missing contract:
  `"gate route-write not implemented: 'application.deploy' — no verified flint-gate admin write endpoint (GateAdmin exposes list_routes only)"`.
- Behavior is unchanged: still `Err(PortError::Downstream(...))`, **never `Ok`**.
- Regression test asserts it still refuses + the message names the dependency.

## tests.rs 500-line resolution (blocking, do before adding tests)

`crates/fpa-app/src/task_runner/tests.rs` is **597 lines — already over the 500
hard limit** before any p14 test. Resolution: convert the inline file-module into a
**directory module**:

- `task_runner/tests/mod.rs` — shared fakes + helpers + all existing non-MCP tests
  and the `every_*_dispatched` guards. Declares `mod mcp;`.
- `task_runner/tests/mcp.rs` — the MCP tests (moved `mcp_tool_list_*` +
  `every_mcp_catalog_kind_is_dispatched`) plus the new p14 c001 invoke tests. A
  recording `FakeMcp` (captures `name`+`arguments`) lives here.

`mod.rs`'s `mod tests;` declaration is unchanged (Rust resolves `tests/mod.rs`).
Both files land under 500 lines.

## Integration milestone

One `cargo test -p fpa-app` after BOTH changes are fully implemented:
- c001: catalog invariants (3), extended `every_mcp_catalog_kind_is_dispatched`,
  `mcp.tool.call` invoke-passthrough (name+arguments forwarded), sub-operator denied
  before any port call, unknown-Mcp-kind refusal, arguments-not-logged.
- c002: `application.deploy` still-refuses + message-marker regression.

Then `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all`.
Reserve the other 2 test runs for real re-runs.

## QA gates

- c001 — artifact-refiner **runs** (behavioral, 3+ files touched).
- c002 — artifact-refiner **may be skipped** (<3 files, message + docs).

## Guardrails

No new port (`McpClient::call_tool` + `fpa-mcp` adapter already real). No sibling-repo
edit. No `unwrap`/`expect` in lib crates (test code may `expect`). `#[non_exhaustive]`
public enums. ≤500 lines/file. Never log MCP `arguments`, tokens, or claims.
`application.deploy` never returns `Ok`.
