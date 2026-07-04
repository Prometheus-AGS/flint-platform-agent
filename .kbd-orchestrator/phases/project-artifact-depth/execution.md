# Execution — project-artifact-depth

**Backend:** OpenSpec (KBD source of truth). **Dispatch:** in-session.
**Philosophy:** implementation-first — c001+c002 together (same files), ONE batched
`cargo check`, fast unit tests; c003 mechanical CLI.

## Order & contract

1. p7-c001 richer-project-create + `TargetPort::Store` (catalog.rs + task_runner.rs).
2. p7-c002 application.define store home (same files; uses the Store arm).
   → **section boundary: one `cargo check`** + clippy + fmt.
   → `rust-reviewer` on the store-dispatch changes.
3. p7-c003 archive p1–p4: reconcile 2 partials, archive the 12 complete, verify.

## Test-wait budget

≤3 full `cargo test`. Planned: **1** run to confirm the new unit tests green (all
in-mem store; no Docker, no heavy deps). 2 held in reserve.

## QA gate

artifact-refiner absent. c001+c002 get a `rust-reviewer` pass (validation-before-
write + new enum variant + upsert correctness). c003 doc/CLI-only (skip QA).
