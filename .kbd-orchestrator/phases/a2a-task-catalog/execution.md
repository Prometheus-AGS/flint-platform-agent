# Execution — a2a-task-catalog

**Phase:** a2a-task-catalog
**Stage:** execute
**Date:** 2026-07-08
**Backend:** `openspec`
**Input:** `plan.md`, `plan.handoff.json`

---

## Backend selection

**Chosen: `openspec`.** The plan seeded three spec-backed changes
(`p13-c001`, `p13-c002`, `p13-c003`), each with `proposal.md` + `tasks.md` +
a `specs/task-catalog/spec.md` delta, all `openspec validate`-clean. Spec-backed
traceability is the established convention for this repo (every prior phase used
OpenSpec), and each change carries requirement/scenario deltas the Reflect stage
verifies against. `native-tool`/`hybrid`/`manual` do not apply — there is no
external tool decomposition and the work is fully automatable in-repo.

## Dispatch contract

Task execution is driven by **`/kbd-apply`** (the KBD-owned apply driver that
fires per-task `task:before`/`task:after` hooks and emits position signals).
Bare `/opsx:apply` is **not** used — it has no KBD awareness (no hooks, no
`progress.json`, no waypoint).

**Order is load-bearing — apply strictly in sequence:**

| N | Change | Tasks | Depends on | Agent | QA gate |
|---|---|---|---|---|---|
| 1 | `p13-c001-split-task-runner` | 8 | — | rust-build-resolver | skip (mechanical, ≤2 files by move) |
| 2 | `p13-c002-forge-describe-name` | 10 | c001 | tdd-guide | skip (<3 files) |
| 3 | `p13-c003-gate-mcp-read-kinds` | 13 | c001 | tdd-guide | run (catalog + runner + tests) |

c002 and c003 are independent of each other but both build on c001's directory
split; apply c002 then c003 for a clean linear history.

### Per-change apply steps

For each change, in order:

1. `/kbd-apply <change-id>` — walk `tasks.md`, firing `task:before`/`task:after`
   per task; update `progress.json` (`active_change`, then `changes_completed`).
2. On all tasks `[x]`: mark the change `DONE` in `progress.json`.
3. **QA gate** (see table): skip for c001 (mechanical move, behavior-preserving)
   and c002 (<3 files); **run** artifact-refiner for c003 (touches catalog.rs +
   task_runner/mod.rs + task_runner/tests.rs). On PASS → `/opsx:verify` →
   `/opsx:archive`. On FAIL → mark BLOCKED, `/refine-code`.
4. Advance to the next change.

## Implementation-first execution posture (operator workflow — overrides test-as-you-go)

Per the repo's Fast Iteration Workflow and the plan:

- **Implement c001 → c002 → c003 fully first**, then write/run the integration
  tests. Do not compile after every file.
- `cargo check -p fpa-app` **sparingly, at section boundaries** (after each
  change's code is written together), not per file.
- **≤3 `cargo test` runs total for the whole phase.** The single required test
  milestone is `p13-c003` task 4.3 (`cargo test -p fpa-app`), which exercises the
  c001 moved tests + c002 argument-asserting test + c003 catalog/guard tests in
  one run. Reserve the other two waits for genuine failures only.
- Before done: `cargo clippy --workspace --all-targets -- -D warnings` + `cargo
  fmt --all` clean.

## Invariants held during execution (from plan + Base Rules)

- **Agent-only:** no sibling repo edit (`../flint-forge`, `../flint-realtime-fabric`,
  `../flint-gate` are read-only reference), no fabric dependency.
- **Reads-only:** `mcp.call_tool` + `forge.create_entity` stay out of the catalog;
  `application.deploy` stays refused.
- **No new ports:** all three seams (`describe_table`, `list_routes`, `list_tools`)
  already exist.
- **flint-gate is the only auth boundary** — the agent reads gate via the admin-port
  adapter, never Ory.
- No `unwrap`/`expect` in the lib (`thiserror`); `#[non_exhaustive]` + newtype IDs
  preserved; no file over 500 lines (the reason c001 goes first); never log secrets;
  edition 2024 / toolchain 1.93.

## First pending change

`p13-c001-split-task-runner` — dispatch with `/kbd-apply p13-c001-split-task-runner`.
