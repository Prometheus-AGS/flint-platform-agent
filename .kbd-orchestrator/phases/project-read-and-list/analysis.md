# Analysis — project-read-and-list (SKIPPED)

**Phase:** project-read-and-list
**Date:** 2026-07-04
**Status:** **SKIPPED** — `/kbd-analyze --skip "in-house store reads over existing
ProjectStore; no external deps or research"`

---

## Why skipped

All goals are in-house work over the **existing** `ProjectStore` port and its two
adapters — no library, framework, or skeleton is a candidate, so the tiered pipeline
would return nothing:

- **`project.inspect` → store** — a retarget + a `dispatch_store` arm calling the
  existing `store.get`. No new code beyond routing.
- **`project.list` → store** — one new port method `ProjectStore::list()`, trivially
  implemented over the in-mem `HashMap` (`values().cloned()`) and the Pg table
  (`SELECT body`, mirroring the existing `get`). No new crate; `tokio-postgres` /
  `deadpool` are already present from p6.
- **p6 archival** — the `openspec` CLI (already in use).

Assess resolved every question with a sensible lean (below); each is decidable at
spec, none needs external evidence or an operator gate.

## Design leans carried to spec (from assess §4)

1. **`project.list` pagination** → **full list now** (YAGNI; small store; paginate
   when a real dataset demands it).
2. **`project.inspect` shape** → **whole aggregate** (the read of what `create` wrote).
3. **List ordering** → unordered acceptable; if a test wants determinism, **sort by
   id** in the handler (cheap, deterministic — Base Rule 35).
4. **Per-operator visibility/RLS** → return **all** this phase (single-tenant store,
   no ownership column); note per-operator scoping as future debt. gate/forge remain
   the authz authority per the cross-plane contracts.
5. **Pg `list` test** → assert it inside the existing `#[ignore]`d container test
   (Docker down); the in-mem `list` is the runnable proof this session.

## Candidates

None. `library-candidates.json` records zero candidates; the gaps are `build_required`
(in-house).

## Handoff to spec

Analyze skipped (justified — no external deps). Spec from the assessment + the five
leans: retarget `project.inspect`/`project.list` to `Store`; add `ProjectStore::list()`
to the port + both adapters; full unordered-or-sorted list; single-tenant; p6
archival.
