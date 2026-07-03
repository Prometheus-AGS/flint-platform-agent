# Plan — project-artifact-depth

**Phase:** project-artifact-depth
**Backend:** OpenSpec (`openspec/changes/p7-c00N-*`)
**Changes:** 3 (all validate `--strict`)
**Working philosophy:** implementation-first — build c001+c002 together (same file:
`task_runner.rs`/`catalog.rs`), ONE batched `cargo check`, then the fast unit tests;
c003 is mechanical CLI.

---

## Ordered change list

### 1. p7-c001-richer-project-create  (G1 + G4) — **first**
Richer `project.create` (serde-validated nested aggregate; server owns
id/schema_version) + `TargetPort::Store` routing.
- **Why first:** it introduces `TargetPort::Store`, which c002 depends on.
- **Depends on:** nothing (domain types + `ProjectStore` port already exist).
- **Recommended agent:** `rust-reviewer` after implementation (validation-before-write
  + a new enum variant + routing — wants a correctness pass). No security-review (no
  auth/secret surface changes).

### 2. p7-c002-application-define-home  (G3) — **second**
`application.define` store-backed: load project → upsert `ApplicationDef` by id → put;
unknown project rejected.
- **Why second:** hard-depends on c001's `TargetPort::Store` + the store dispatch arm.
- **Depends on:** **c001 (hard).**
- **Recommended agent:** covered by the same `rust-reviewer` pass (same file/pattern).

--- SECTION BOUNDARY: one batched `cargo check` across the workspace here ---
(c001 + c002 are the same-file store-dispatch work — implement both, then check once.)

### 3. p7-c003-archive-p1-p4  (G5) — **last**
Archive the 12 complete p1–p4 changes into `specs/`; reconcile-or-skip the 2 partials.
- **Why last:** independent housekeeping; separated from code (Base Rule 31) and
  placed after so no archive/edit race on `openspec/`.
- **Depends on:** nothing.
- **Recommended agent:** none — mechanical CLI + a `git status` / `openspec list`
  sanity check.

---

## Dependency graph

```
c001 (TargetPort::Store + richer create)
  └─► c002 (application.define — needs Store arm)   [hard]
c003 (archive p1–p4)                                 [independent; last]
```

## Execution contract (implementation-first)

- Implement **c001 + c002 together** in `catalog.rs` + `task_runner.rs` (they share
  the `TargetPort::Store` variant and the store dispatch arm) before compiling.
- One batched `cargo check` at the section boundary + clippy + fmt.
- **Fast unit tests only** (all in-mem `ProjectStore` — no Docker, no heavy suite):
  nested-payload store, malformed-reject, schema_version-server-owned, Store-routing,
  application upsert, unknown-project reject. **0–1 `cargo test` waits** (one run to
  confirm green; ≤3 budget, well within).
- c003: reconcile the 2 partials (read the trailing task), archive the complete set,
  verify with `openspec list` + `git status`.

## Compliance notes

- **Immutability (coding rules):** the `application.define` upsert builds a new
  `applications` vec / new `Project` — no in-place mutation.
- **Base Rule 39 (structured artifacts):** server owns `schema_version`; the client
  cannot set it (spec'd + tested).
- **`#[non_exhaustive]`:** the new `TargetPort::Store` variant keeps the enum's
  attribute; cross-crate matches already use wildcards.
- No new deps. No gateway-surface change (same task kinds, richer input).

## Handoff to execute

3 validated OpenSpec changes. c001 (richer create + `TargetPort::Store`) and c002
(application.define store home) are the same-file store-dispatch work — build both,
one batched check, fast unit tests. c002 hard-depends on c001. c003 (archive p1–p4)
is independent housekeeping, last. First change to apply: **p7-c001-richer-project-
create**.
