# Execution — live-smoke-and-durable-store

**Backend:** OpenSpec (KBD source of truth; tasks in `openspec/changes/p6-c00N-*/tasks.md`).
**Dispatch:** in-session, agent-implemented.
**Philosophy:** implementation-first — build c001 whole, ONE batched `cargo check`,
fast tests only; c002 is mechanical CLI.

## Order & contract

1. p6-c001-durable-project-store — new `fpa-store-pg` crate (tokio-postgres +
   deadpool), JSONB store, config + composition-root selection.
   → **one batched `cargo check`** + clippy + fmt.
   → fast tests: `Project`↔JSONB round-trip + in-mem path.
   → `testcontainers` restart test **`#[ignore]`d** (Docker down) — compiled, NOT run.
   → `rust-reviewer` (hexagonal boundary + error mapping).
2. p6-c002-archive-p5-changes — `openspec archive` p5-c001..c004; verify.

## Test-wait budget

≤3 full `cargo test`. Planned: **0 heavy runs** — only the fast unit suite, run once
to confirm green. The durable-store restart proof is deferred (Docker unavailable) and
does NOT consume a wait.

## Honest constraint

The durable store ships **compiled + unit-tested** but **not proven against a real
Postgres** this session (Docker daemon unreachable). The `#[ignore]`d container test is
the proof, to run when the stack is up. Stated in tasks + reflection — not silent.

## QA gate

artifact-refiner absent (all phases). c001 gets a `rust-reviewer` pass. c002 is
doc/spec-only (skip QA).
