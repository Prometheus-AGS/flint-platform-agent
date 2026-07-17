# Reflection — a2a-task-catalog

**Phase:** a2a-task-catalog (`p13`)
**Backend:** OpenSpec
**Closed:** 2026-07-08
**Changes:** 3/3 applied, verified, archived (`openspec/changes/archive/2026-07-08-p13-c00{1,2,3}-*`)

This was a **manual pivot phase**: the prior `realtime-receipt` reflection auto-seeded
`realtime-receipt-unblock`, but that is BLOCKED on upstream `flint-realtime-fabric#2`
(re-verified OPEN). Rather than idle-block, we advanced the alternative that reflection
itself named — expanding and correcting the agent-owned A2A administrative task catalog.
Agent-only work: no sibling repo touched, no fabric dependency.

## Goal Achievement

| Goal | Verdict | Evidence |
|---|---|---|
| **G1 — Fix `forge.table.describe` to honour its input** | **MET** | `dispatch_forge` now calls `parse_table_name(input)?` → `describe_table(name, bearer)` instead of the `"<unspecified>"` placeholder (c002, `task_runner/mod.rs`). Test `describe_threads_validated_table_name_to_forge` asserts `FakeForge.described == Some("widgets")`; `describe_rejects_empty_table_name` asserts an empty/whitespace `name` is refused before the port call. |
| **G2 — Wire real `GateAdmin` read into catalog** | **MET (with deliberate role change)** | Added `gate.route.list` → `TargetPort::Gate`, dispatching to `self.gate.list_routes()` (not the `{"routes":[]}` stub). **Role shipped as `operator`, not the proposed `viewer`** — see Delta 1. Covered by `gate_route_list_runs_as_operator_and_lists_routes` + `gate_route_list_denies_non_operator`. |
| **G3 — Wire real `McpClient` read into catalog** | **MET** | Added `mcp.tool.list` → `TargetPort::Mcp` / `viewer`, dispatching to `self.mcp.list_tools()` (existing arm confirmed, no mod.rs change). `call_tool` correctly kept out of the catalog. Covered by `mcp_tool_list_runs_as_viewer_and_lists_tools` + new `every_mcp_catalog_kind_is_dispatched` guard. |
| **G4 — Read-vs-write honesty in `dispatch_forge`** | **MET** | Preserved: unmapped forge kinds still return a clean `Downstream(...)` rather than a silent read fallback; both new reads are *declared* catalog entries with schemas, never implicit fallthroughs. `application.deploy` remains a refused write (`gate_write_kind_refuses_without_listing_routes` green). No fabricated data, no empty-stub-as-success. |
| **G5 — Catalog invariants stay green** | **MET** | `lookup_known_and_unknown`, `kinds_are_unique`, `every_entry_has_valid_input_schema` all pass with the two new entries. New guard `every_mcp_catalog_kind_is_dispatched` extends the same silent-drop protection to Mcp. |

**Goal achievement: 5/5 MET (100%).**

## Success Criteria

1. `forge.table.describe` provably describes the requested table — **YES** (input-reaches-port test).
2. `gate.route.list` + `mcp.tool.list` in `CATALOG`, dispatch to real ports, covered by runner tests with fakes asserting the call — **YES**.
3. `dispatch_gate` / mcp arm no longer return hardcoded empty stubs for the cataloged reads — **YES**.
4. All catalog invariant tests pass; `cargo check --workspace` clean; `clippy --workspace --all-targets -D warnings` clean — **YES**.

## Delivered Changes

| Change | Scope | Files | Result |
|---|---|---|---|
| **c001-split-task-runner** | Mechanical: split the 828-line `task_runner.rs` (over the 500-line CI gate) into `task_runner/{mod.rs (~341), tests.rs (~490)}`, both under the limit. Behavior-preserving. | 1 (→ dir module) | archived |
| **c002-forge-describe-name** | G1 bug fix: thread validated `name` through `dispatch_forge`; add `parse_table_name` guard + 2 tests. | 2 (mod.rs, tests.rs) | archived |
| **c003-gate-mcp-read-kinds** | G2+G3+G5: two read catalog entries; `GATE_READ_KINDS` classification; `every_mcp_catalog_kind_is_dispatched` guard; 4 behavioral/guard tests. | 2 (catalog.rs, tests.rs) | archived |

## Verification Milestone

Per the operator's implementation-first workflow (≤3 `cargo test` runs/phase), all
assertions were batched into a **single** integration milestone at c003 task 4.3:

- `cargo test -p fpa-app` → **39 passed; 0 failed; 0 ignored** (run #1 of ≤3) — covers c001's moved tests, c002's argument-threading test, and c003's catalog/guard/behavioral tests in one pass.
- `cargo clippy --workspace --all-targets -- -D warnings` → clean.
- `cargo fmt --all --check` → clean.

Two of the three ≤3 test-run budget slots were left unspent — the front-loaded coding
rules (strong types, no `unwrap` in libs, hexagonal boundaries) held, and holistic
integration validated on the first run.

## Artifact Quality Summary

| Metric | Value |
| --- | --- |
| Changes with QA (artifact-refiner) | 0/3 |
| First-pass pass rate | n/a (no QA runs) |
| Changes requiring refinement | 0 |
| Total refinement iterations | 0 |

**All three changes qualified for the QA skip rule** (`kbd-execute` → "fewer than 3 files
modified"): c001 mechanical (1 file), c002 (2 files), c003 (2 files). No `.refiner/`
logs exist. See Delta 2 — c003 was *planned* as QA-RUN but the authored change came in
under the file-count threshold, so the skip rule governed. Correctness was instead
carried by the invariant tests + the single integration milestone, not by the refiner.

### Recurring Constraint Violations

None. No constraint failures across any change.

## Deltas (plan/goal vs. as-built)

**Delta 1 — G2 role: `viewer` (proposed) → `operator` (shipped).**
- *What:* the goal proposed `gate.route.list` as `viewer` but explicitly said "viewer is the floor — confirm against the role model". The plan's resolution #1 raised it to `operator`.
- *Root cause:* gate routes expose operational *topology* (upstreams, auth pipelines, streaming config) — an information-disclosure surface, unlike `forge.table.list` whose table *names* are benign. Base Rule 33 (security not optional; least-privilege on reads that leak structure): default the role **up** on topology-revealing reads.
- *Status:* resolved, not a miss. Documented inline in `catalog.rs` and in the c003 spec ("enumerates gate routes for operators"). This is the correct security posture and should be the house rule for future topology reads.

**Delta 2 — c003 QA: planned RUN → actual SKIP.**
- *What:* the plan/execute handoff tagged c003 "QA-RUN artifact-refiner" (anticipating 3+ files); as authored it touched only 2 source files (mod.rs needed no change since the Mcp dispatch arm and gate write-classifier already existed).
- *Root cause:* the pre-planned QA flag was a size estimate; the deterministic `<3-file` skip rule is authoritative and overrode it once the real diff was known.
- *Status:* acceptable. The skip rule is the governing contract; the file-count estimate was conservative. Correctness verified by the test milestone + invariant guards regardless.

**Delta 3 — G2/G3 needed no new `dispatch` arms in `mod.rs`.**
- *What:* c003 tasks 2.1/2.2 turned out to be *confirmations* — `is_gate_write_kind` already matched only `application.deploy` (so `gate.route.list` falls to `list_routes()`), and the `TargetPort::Mcp => self.mcp.list_tools()` arm already existed.
- *Root cause:* the runner's dispatch was already port-complete; only the *catalog* (the kind→port→role table) was missing the entries. Confirms the catalog-driven design: adding a read kind is a catalog edit, not a runner edit, when the port arm exists.
- *Status:* positive — validates the intended architecture (both protocol surfaces auto-consume `CATALOG`; no gateway edits to add kinds).

## Technical Debt

- **None introduced.** No new ports, no new dependencies, no files over 500 lines (c001 specifically retired the one carried >500-line file). No `unwrap`/`expect` in libs; `#[non_exhaustive]`/newtype IDs preserved.
- **Carried/deferred (by design, not debt):** `mcp.call_tool` (write/invoke) and `forge.create_entity` remain out of the catalog; `application.deploy` remains a refused write stub. These are deliberate scope boundaries (reads-first), re-openable when a write-catalog phase is scheduled.

## Lessons Captured

1. **Topology-revealing reads default their role UP.** `gate.route.list` = operator, not viewer, even though it is "just a read". Route/upstream/auth-pipeline listings are an information-disclosure surface. Contrast: `forge.table.list` = viewer (table names are benign). Encode this as the standing rule for future gate/infra reads. → house rule.
2. **Adding a task kind is a catalog edit, not a runner edit** — when the target port's dispatch arm already exists. The catalog (`kind → TargetPort → role → schema`) is the single source of truth; both A2A and MCP surfaces auto-consume it. This is the payoff of the port-complete-runner + data-driven-catalog split.
3. **The deterministic QA skip rule (`<3 files`) beats a pre-planned QA flag.** Size the QA gate from the *actual* diff, not the plan's estimate. When a change lands smaller than forecast, the file-count rule governs; verification shifts to invariant tests + the integration milestone.
4. **Implementation-first + front-loaded rigor spent 1 of 3 test slots.** Batching all three changes' assertions into one `cargo test` at the final milestone (39/39 green first try) confirms the workflow: trust the coding rules during implementation, validate holistically at the end.
5. **A "confirm before finalising" clause in a goal is a real fork, not boilerplate.** G2 said "confirm against the role model" and the answer changed the shipped role. Treat such clauses as decisions to resolve in the plan, with rationale — which is exactly what happened here.

## Recommended Next Phase

The parked **`realtime-receipt-unblock`** remains BLOCKED on upstream `flint-realtime-fabric#2`
— do NOT re-enter until that issue closes. Two agent-only alternatives, in priority order:

1. **`a2a-task-catalog-writes`** *(recommended)* — the natural continuation. This phase
   deliberately scoped writes out (reads-first). Now catalog the write/invoke paths that
   already have real ports: `mcp.tool.call` (`McpClient::call_tool`) and, if the role model
   permits, promote `application.deploy` from stub to a real gate write via the admin port.
   Same agent-only, no-fabric, no-new-port discipline; the read work here is the template.
   Writes need stricter role gating (admin floor) and idempotency/audit consideration
   (Base Rules 34/35) — a meatier but well-bounded phase.

2. **`gate-admin-surface-expansion`** — broaden the gate administrative surface beyond
   `list_routes` (route CRUD, auth-provider inspection) via gate's admin port only, keeping
   flint-gate as the sole auth boundary. Larger; depends on gate's admin API stability.

Recommendation: **`a2a-task-catalog-writes`** — it directly completes the catalog the last
three phases have been building, reuses this phase's exact patterns, and stays fully
agent-owned while fabric#2 is out.
