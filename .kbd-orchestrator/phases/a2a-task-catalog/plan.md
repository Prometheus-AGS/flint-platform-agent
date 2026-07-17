# Plan — a2a-task-catalog

**Phase:** a2a-task-catalog
**Stage:** plan
**Backend:** OpenSpec (`openspec/changes/`) — phase number `p13`
**Input:** `assessment.md` (verdict ACTIONABLE — 3 gaps, all in `crates/fpa-app/src/{catalog.rs, task_runner.rs}`)

---

## Resolutions to the assessment's open questions

1. **Role for `gate.route.list` → `operator`** (raised from the viewer default).
   Gate routes expose operational *topology* — upstreams, auth pipelines, streaming
   config — not benign metadata. `forge.table.list` is `viewer` because table *names*
   are harmless; a route list is an information-disclosure surface. When in doubt on a
   read that reveals topology, default the role **up**. (Base Rule 33: security is not
   optional; least-privilege on reads that leak structure.)
2. **Runner split → tests-only extraction** into `task_runner/tests.rs`. Production code
   is lines 1–341; the `#[cfg(test)] mod tests` block is 342–828 (487 lines). Moving the
   test module to a sibling file leaves the prod file at ~341 lines and the test file at
   ~490 — both under the 500-line CI gate — with a purely mechanical, behavior-preserving
   diff. (Base Rule 2/3: smallest surgical change; split-by-plane is deferred YAGNI.)
3. **Add `every_mcp_catalog_kind_is_dispatched` guard → yes.** Low-cost symmetry with the
   existing Gate/Store guards; closes the same silent-drop class for Mcp that G3 introduces.

## Ordered change list

| # | Change | Target port(s) | Backend | Agent | Depends on |
|---|---|---|---|---|---|
| 1 | **p13-c001-split-task-runner** | — (mechanical) | OpenSpec | rust-build-resolver | — |
| 2 | **p13-c002-forge-describe-name** | Forge (`describe_table`) | OpenSpec | tdd-guide | c001 |
| 3 | **p13-c003-gate-mcp-read-kinds** | Gate (`list_routes`), Mcp (`list_tools`) | OpenSpec | tdd-guide | c001 |

Ordering rationale: **c001 first** so the file is compliant (<500 lines) *before* c002/c003
add dispatch arms + tests to it — otherwise those changes knowingly ship a file over the
CI-enforced gate. c002 and c003 are independent of each other (different dispatch arms,
different ports) but both build on the split file; apply c002 then c003 for a clean linear
history. All three are agent-only — no sibling repo, no fabric dependency, no new ports.

## Change details

### c001 — split `task_runner.rs` into a directory module

- **Why:** `task_runner.rs` is 828 lines, already over the 500-line hard limit (carried
  debt). Adding two dispatch arms + integration tests would push it further.
- **What:** Convert `crates/fpa-app/src/task_runner.rs` → `crates/fpa-app/src/task_runner/`
  with `mod.rs` (the 1–341 production body, verbatim) and `tests.rs` (the 342–828
  `#[cfg(test)]` block, moved out). `mod.rs` declares `#[cfg(test)] mod tests;`. The test
  module gains `use super::*;` (it already references `TaskRunner`, the fakes, `catalog`,
  `TargetPort`, etc. — all in scope via `super`). No production behavior changes; no public
  API changes; `crate::task_runner::TaskRunner` still resolves.
- **Verify:** `cargo check -p fpa-app` green; the moved tests still compile and pass
  (deferred to the ≤3 `cargo test` budget). File-length: both files < 500.

### c002 — fix `forge.table.describe` to thread the validated `name` (G1)

- **Why:** `dispatch_forge` at `task_runner.rs:154` calls
  `self.forge.describe_table("<unspecified>", bearer)` — the schema-validated `name` input
  is silently discarded. The `ForgeMetadata::describe_table(name, bearer)` port takes the
  real name. The bug is untested: `valid_required_input_passes` only asserts the port was
  *called*, never *with what*.
- **What:** Extract `name` from the validated task input (the `SCHEMA_TABLE_NAME` contract
  already requires `table` as a non-empty string) and pass it to `describe_table`. Mirror
  the existing input-extraction pattern in `dispatch_store` (`project.inspect` parses
  `project_id` the same way). On a missing/empty `table` after schema validation, return
  `AppError::InvalidInput` (defense in depth — schema should already reject it).
- **What (test):** Extend `FakeForge` to capture the requested table name (e.g.
  `described: Mutex<Option<String>>` or an `Atomic…`-backed capture), and add/upgrade a
  test asserting `describe_table` is called **with the exact requested name**, not a
  placeholder. This is the argument-asserting test the assessment called for.
- **Verify:** unit test proves the threaded name reaches the port; no forge write path
  touched; `cargo check/clippy/fmt` green.

### c003 — add `gate.route.list` + `mcp.tool.list` read kinds (G2 + G3)

- **Why:** `dispatch_gate` already calls `list_routes` for non-write kinds, and the runner
  already has a `TargetPort::Mcp => self.mcp.list_tools()` arm — but **no catalog kind
  targets Gate-read or Mcp**, so both paths are unreachable. `GATE_READ_KINDS = &[]` is an
  explicit empty placeholder waiting for the Gate read kind.
- **What (catalog):** Add two `CatalogEntry`s to `CATALOG` in `catalog.rs`:
  - `gate.route.list` → `TargetPort::Gate`, `required_role: "operator"`, `SCHEMA_EMPTY`.
  - `mcp.tool.list` → `TargetPort::Mcp`, `required_role: "viewer"`, `SCHEMA_EMPTY`.
  Both surfaces (A2A `submit`, MCP `tool_definitions`) read `catalog::CATALOG`, so the new
  kinds auto-appear — **no gateway edits.**
- **What (runner):** `gate.route.list` is a *read*, so `is_gate_write_kind` must keep
  returning `false` for it (it already does — only `application.deploy` is a write); no
  dispatch change needed beyond confirming the read branch returns `list_routes()`. Confirm
  `dispatch` routes `TargetPort::Mcp` to the existing `list_tools()` arm for `mcp.tool.list`.
  Register `gate.route.list` in `GATE_READ_KINDS` so `every_gate_catalog_kind_is_classified`
  passes.
- **What (guards):** Add a new `every_mcp_catalog_kind_is_dispatched` guard test
  (`MCP_KINDS = &["mcp.tool.list"]`) mirroring the Gate/Store guards, so a future Mcp kind
  that isn't dispatched fails loudly.
- **Verify:** the 3 catalog tests (`lookup_known_and_unknown`, `kinds_are_unique`,
  `every_entry_has_valid_input_schema`) auto-cover the new entries; `gate.route.list` runs
  → `list_routes` called, no write refusal; `mcp.tool.list` runs → `list_tools` called; the
  Gate guard passes with the updated `GATE_READ_KINDS`; the new Mcp guard passes.

## Read-vs-write honesty (G4) — a standing invariant, not a change

Every new kind is a **declared** catalog entry with a real dispatch arm — never a
fallthrough. Writes stay deferred and honestly refused: `application.deploy` (refused at
gate), `mcp.call_tool`, `forge.create_entity` remain **out of scope** this phase. No
empty-stub-as-success anywhere. (Base Rule 2 + assessment G4.)

## Execution posture (operator workflow)

Implement c001 → c002 → c003 fully first, then the integration tests. `cargo check -p
fpa-app` at section boundaries; ≤3 `cargo test` runs total for the phase; `cargo clippy
--workspace --all-targets -- -D warnings` + `cargo fmt --all` before done. No `unwrap`/
`expect` in the lib; `#[non_exhaustive]`/newtype discipline preserved; never log secrets.
flint-gate stays the only auth boundary — the agent reads gate via the admin-port adapter,
never Ory.

## First change to apply

`p13-c001-split-task-runner` (mechanical runner split — unblocks the compliant file that
c002/c003 extend).
