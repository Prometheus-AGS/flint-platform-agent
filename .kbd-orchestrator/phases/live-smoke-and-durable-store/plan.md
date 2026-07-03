# Plan — live-smoke-and-durable-store

**Phase:** live-smoke-and-durable-store (re-scoped: durable store + p5 archival)
**Backend:** OpenSpec (`openspec/changes/p6-c00N-*`)
**Changes:** 2 (both validate `--strict`)
**Working philosophy:** implementation-first — implement c001 fully, ONE batched
`cargo check`, then the fast tests; c002 is a mechanical CLI step, no code.

---

## Ordered change list

### 1. p6-c001-durable-project-store  (G2) — **first**
New `fpa-store-pg` adapter crate: `ProjectStore` over `tokio-postgres` 0.7.18
(`library: cand-001`) + `deadpool-postgres` 0.14.1 (`cand-002`); `Project` stored
whole as JSONB in an agent-owned table; composition root selects Pg-vs-InMem by
`FPA_PROJECT_DB_URL`. Restart-survival test via `testcontainers` 0.27 (`cand-003`),
`#[ignore]`d (Docker down here).
- **Why first:** it's the phase's substance and the only code change. Touches a new
  crate + `fpa-gateway` config/state.
- **Depends on:** nothing (the `ProjectStore` port already exists from p5-c001).
- **Recommended agent:** `rust-reviewer` after implementation (a new infra adapter
  crate — wants a hexagonal-boundary + error-mapping review); no security-review
  needed unless the DB URL handling looks risky (it's redacted by spec).

### 2. p6-c002-archive-p5-changes  (G4) — **last**
`openspec archive` p5-c001..c004 into `specs/`.
- **Why last:** pure housekeeping; ordering it after c001 keeps the spec-baseline
  move separate from the code change (Base Rule 31 — mechanical vs behavioural
  changes separated) and avoids any archive/edit race on `openspec/`.
- **Depends on:** nothing (p5 changes are already shipped + validated).
- **Recommended agent:** none — mechanical CLI + a `git status` sanity check.

---

## Dependency graph

```
c001 (durable store — code)
c002 (archive p5 — housekeeping)   [independent; sequenced last]
```

No hard dependency between them; c002 is placed last so it captures the shipped
baseline cleanly and never races the c001 crate work.

## Execution contract (implementation-first)

- Implement **c001 in full** (new crate: Cargo.toml + schema.sql + `PgProjectStore` +
  config + state wiring) before compiling. One batched `cargo check` at the crate
  boundary.
- **Fast tests only** run this session: `Project` ↔ JSONB serde round-trip + the
  existing in-mem path. The `testcontainers` restart-survival test is **`#[ignore]`d**
  (Docker unavailable) — implemented and compiled, but NOT run; this is stated, not
  silently green. **0 of 3 test-waits** are spent on a heavy `cargo test` unless the
  fast suite needs a run to confirm green.
- c002: run the four `openspec archive` commands, verify with `openspec list` +
  `git status` (openspec-only moves).

## Compliance notes

- **Hexagonal (Rule 16):** `fpa-store-pg` is a NEW adapter crate; `fpa-app` does NOT
  depend on it. Only `fpa-gateway` (composition root) imports it.
- **New deps verified (Rule 22):** tokio-postgres 0.7.18, deadpool-postgres 0.14.1,
  testcontainers 0.27.3 / -modules 0.15.0 (Tier-3 registry, this phase's analyze).
- **Secrets:** `FPA_PROJECT_DB_URL` redacted in `Debug`; never logged. No
  `unwrap`/`expect` in the adapter lib (`thiserror`/`PortError` mapping).
- **No silent green:** the ignored container test is documented as not-run-this-session
  in the change tasks and the reflection.

## Handoff to execute

2 validated OpenSpec changes. c001 (durable `fpa-store-pg` adapter — the code) first,
c002 (archive p5 into `specs/` — housekeeping) last. First change to apply:
**p6-c001-durable-project-store**. Execute implementation-first: build the crate
whole, one batched check, run the fast tests; the `testcontainers` restart test is
`#[ignore]`d (Docker down) — compiled but not run, stated honestly.
