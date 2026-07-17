# Assessment — a2a-task-catalog

**Phase:** a2a-task-catalog
**Stage:** assess
**Date:** 2026-07-08
**Backend:** OpenSpec (`openspec/changes/` — new `pN-cNNN-*` changes to be seeded at Plan)
**Verdict: ACTIONABLE — a small, honest, agent-only surface. 3 concrete gaps, all in two files.**

---

## Scope recap

Expand + correct the A2A administrative task catalog. All work lives in
`crates/fpa-app/src/{catalog.rs, task_runner.rs}`. No sibling repo, no fabric
dependency — this is the deliberate pivot away from the BLOCKED
`realtime-receipt-unblock` phase (upstream `flint-realtime-fabric#2` still OPEN).

## How the surface actually wires (grounded — corrects pre-pivot notes)

The catalog is the single contract; **both** protocol surfaces already read it,
so no route/handler edits are needed for new kinds:

| Surface | File | Wiring |
|---|---|---|
| A2A `POST /a2a/tasks` | `fpa-gateway/src/routes/a2a.rs:44` `submit` | **Fully wired** (not a stub, despite the module doc-comment): builds `AdminTask` from body, runs the real `TaskRunner`. |
| MCP `tools/list` + `tools/call` | `fpa-gateway/src/routes/mcp.rs:121` `tool_definitions` | Iterates `catalog::CATALOG` → one MCP tool per kind, `inputSchema` = `entry.input_schema()`. A new catalog kind auto-appears as an MCP tool. |

So a new kind = a `CatalogEntry` + a `task_runner` dispatch arm + tests.
Nothing in the gateway changes.

## Gap analysis against phase goals

| Goal | Status | Evidence / gap |
|---|---|---|
| **G1** fix `forge.table.describe` ignored input | **REAL BUG — confirmed** | `task_runner.rs:154` calls `self.forge.describe_table("<unspecified>", bearer)` — the validated `name` input is discarded. The `ForgeMetadata::describe_table(name, bearer)` port (`fpa-ports/src/forge.rs:21`) takes the real name. The existing test `valid_required_input_passes` (:522) only asserts the port was *called*, never *with what* — so the bug is untested. **Fix + assert the argument.** |
| **G2** wire `GateAdmin::list_routes` into a catalog kind | **PARTIALLY DONE — just needs the entry** | `dispatch_gate` (:288) **already** calls `self.gate.list_routes()` for non-write kinds — it is *not* a stub. But **no catalog kind targets Gate as a read**, so `list_routes` is currently unreachable (`application.deploy` is the only Gate kind and it's a *write*, correctly refused at :289). And the guard test `every_gate_catalog_kind_is_classified` (:787) has an empty `GATE_READ_KINDS = &[]` placeholder **explicitly waiting for this**. G2 = add `gate.route.list` (Gate/viewer, `SCHEMA_EMPTY`) + register it in `GATE_READ_KINDS`. |
| **G3** wire `McpClient::list_tools` into a catalog kind | **REAL STUB — confirmed** | Runner dispatch `TargetPort::Mcp => self.mcp.list_tools()` (:124) exists but **no catalog kind targets Mcp**, so it's dead. Add `mcp.tool.list` (Mcp/viewer, `SCHEMA_EMPTY`). `call_tool` (a write/invoke) stays deferred. |
| **G4** preserve read-vs-write honesty | **ALREADY HONEST — keep it** | `dispatch_forge` unmapped→`Downstream("write API pending")` (:157); `dispatch_gate` writes→clean refuse (:290); `dispatch_store` unmapped→clean error (:175). No silent empty-stub-as-success anywhere. New kinds must be *declared* entries, never fallthroughs. No change required beyond discipline. |
| **G5** catalog invariants stay green | **NEEDS EXTENSION** | 3 catalog tests (`lookup_known_and_unknown`, `kinds_are_unique`, `every_entry_has_valid_input_schema`) + 2 runner guard tests (`every_gate_catalog_kind_is_classified`, `every_store_catalog_kind_is_dispatched`) exist. New kinds inherit the catalog tests automatically; `GATE_READ_KINDS` must be updated or the gate guard test **fails** (good — it's doing its job). Add a per-kind dispatch assertion for `mcp.tool.list` (there is no `every_mcp_catalog_kind_is_dispatched` guard yet — consider adding one for symmetry). |

## Blocking constraint surfaced by inspection (must shape the plan)

- **`task_runner.rs` is 828 lines — already over the 500-line hard limit** (`catalog.rs` is 172, fine). This is carried debt, not introduced here, but adding two dispatch arms + their integration tests will push it further. **The plan must split `task_runner.rs` into a directory module** (e.g. `task_runner/mod.rs` + extract the `#[cfg(test)]` fakes/tests into `task_runner/tests.rs`, or split dispatch-by-plane) as part of this phase — otherwise we knowingly ship a file that violates a CI-enforced gate. Treat the split as the first change so subsequent arms land in a compliant file. (Base Rule 3: surgical — extract, don't rewrite.)

## Role-model check (G2 open question for Plan)

`gate.route.list` is proposed at **viewer** (it's a read). Confirm against the
role model that a viewer may enumerate gate routes — routes can reveal fabric
topology. If operator-only is required, use `required_role: "operator"`. This is
the one substantive open question; default to **viewer** (consistent with
`forge.table.list`/`fabric.health` reads) unless Plan finds a reason to raise it.
**flint-gate stays the only auth boundary** — the agent reads via the admin port
adapter; it never calls Ory. No change to that posture.

## Non-goals (confirmed against code, Base Rule 2)

- No new ports. G1 reuses `describe_table`; G2 reuses `list_routes`; G3 reuses
  `list_tools`. All three seams already exist.
- No writes: `application.deploy` stays a refused stub; `mcp.call_tool` and
  `forge.create_entity` (a real but uncatalogued write port) stay out of scope.
- No realtime work.

## Recommendation

Proceed to **Plan**. Change ordering (draft):

1. **c001 — split `task_runner.rs`** into a directory module to get under 500
   lines *before* adding arms (mechanical extract; behavior-preserving).
2. **c002 — G1** fix `forge.table.describe` to thread `name`; add an argument-
   asserting test (capture the requested name in `FakeForge`).
3. **c003 — G2 + G3** add `gate.route.list` and `mcp.tool.list` catalog entries;
   update `GATE_READ_KINDS`; add dispatch/guard tests. (Both are read-only
   catalog-entry adds against already-wired dispatch — cohesive.)

Implementation-first per the operator's workflow: implement c001–c003 fully,
then the integration tests, ≤3 `cargo test` runs total. `cargo check --workspace`
at section boundaries; `clippy --workspace --all-targets -D warnings` + `fmt`
before done.

## Open questions for Plan

1. **Role for `gate.route.list`** — viewer (default, matches other reads) vs
   operator (routes reveal topology)? See role-model check above.
2. **Runner split shape** — extract tests-only into `task_runner/tests.rs`
   (smallest diff, fastest to <500) vs split dispatch-by-plane (more cohesive,
   bigger diff)? Recommend the former this phase (Base Rule 2/3).
3. **Add an `every_mcp_catalog_kind_is_dispatched` guard** for symmetry with the
   Gate/Store guards? Low-cost, prevents the same silent-drop class for Mcp.
