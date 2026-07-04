# Plan — project-read-and-list

**Phase:** project-read-and-list
**Backend:** OpenSpec (`openspec/changes/p8-c00N-*`)
**Changes:** 2 (both validate `--strict`)
**Working philosophy:** implementation-first — build c001 whole (port + both adapters
+ dispatch), ONE batched `cargo check`, fast unit tests; c002 mechanical CLI.

---

## Ordered change list

### 1. p8-c001-project-store-reads  (G1 + G2) — **first**
Retarget `project.inspect`/`project.list` to `TargetPort::Store`; new
`ProjectStore::list()` (port + in-mem + Pg); `dispatch_store` inspect/list arms.
- **Why first:** the phase's substance; the only code change.
- **Depends on:** nothing (extends the existing `ProjectStore` from p5/p6).
- **Touches:** `fpa-ports` (trait method), `fpa-app` (`InMemoryProjectStore::list` +
  `catalog.rs` retargets + `task_runner.rs` dispatch + the `STORE_KINDS` test),
  `fpa-store-pg` (`PgProjectStore::list`).
- **Recommended agent:** `rust-reviewer` after implementation (a new port method
  across 2 adapters + dispatch rewiring — wants a correctness + no-forge-call pass).

--- SECTION BOUNDARY: one batched `cargo check` across the workspace here ---

### 2. p8-c002-archive-p6  (G4) — **last**
`openspec archive` p6-c001/c002 into `specs/`.
- **Why last:** independent housekeeping; separated from code (Base Rule 31), after
  so no archive/edit race on `openspec/`.
- **Depends on:** nothing.
- **Recommended agent:** none — mechanical CLI + `git status` / `openspec list` check.

---

## Dependency graph

```
c001 (store reads + list())   [the code]
c002 (archive p6)             [independent; last]
```

## Execution contract (implementation-first)

- Implement **c001 in full** (port method + both adapter impls + catalog retargets +
  dispatch arms + `STORE_KINDS` test update) before compiling. One batched
  `cargo check` + clippy + fmt.
- **Fast unit tests only** (all in-mem `ProjectStore`; no Docker): inspect-existing,
  inspect-unknown, no-forge-call, list-returns-all (deterministic order), empty-list.
  The Pg `list` assertion rides the existing `#[ignore]`d container test (not run —
  Docker down). **0–1 `cargo test` waits** (one run to confirm green; ≤3 budget).
- c002: `openspec archive` p6-c001/c002; verify with `openspec list` + `git status`.

## Compliance notes

- **Hexagonal:** `ProjectStore::list` is a port method; the Pg impl stays in the
  adapter crate; `fpa-app` only touches the in-mem adapter + dispatch. No adapter
  leaks into the app layer.
- **Base Rule 35 (deterministic):** `project.list` result is sorted by id.
- **No forge for project reads:** an explicit test asserts the forge fake is not
  called for `project.inspect`/`project.list`.
- **`#[non_exhaustive]` / integrity test:** the `every_store_catalog_kind_is_dispatched`
  test's `STORE_KINDS` must gain `project.inspect` + `project.list`, else it fails.
- No new deps.

## Handoff to execute

2 validated OpenSpec changes. c001 (store reads + `ProjectStore::list()` across the
port + both adapters + dispatch) is the code — build whole, one batched check, fast
unit tests; the Pg `list` proof rides the `#[ignore]`d container test. c002 (archive
p6) is independent housekeeping, last. First change to apply:
**p8-c001-project-store-reads**.
