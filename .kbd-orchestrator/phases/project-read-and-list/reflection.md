# Reflection — project-read-and-list

**Phase:** project-read-and-list
**Date:** 2026-07-04
**Changes:** 2/2 executed (p8-c001, c002), all CI-green and pushed
(`c23a5cd`, `3f7763a`).

> Sycophancy gate applied. A clean phase — all goals MET, reviewer APPROVE, no
> deferred-runtime caveat. The honest note this time is a **meta** one: three
> consecutive no-infra phases have made the agent's in-memory surface excellent
> while the same infra-gated debt (live smoke, durable-store runtime proof, TLS,
> RLS) keeps accumulating untouched. Flagged for the next-phase decision.

---

## 1. Goal achievement

| Goal | Verdict | Evidence |
|---|---|---|
| `project.inspect` → store read | **MET** | c001: retargeted to `TargetPort::Store`; `store.get` by id; unknown → clean error; **no forge call** (asserted). |
| `project.list` → store (new port method) | **MET** | c001: `ProjectStore::list()` on the port + both adapters; retargeted; deterministic sort by id; empty → `[]`; no forge call. |
| Fold in p6 archival | **MET** | c002: p6-c001/c002 archived; 18 capabilities now baselined. |

**Overall: 100% of in-scope goals MET.** All assess leans held (whole-aggregate
inspect, full list now, sort-by-id, single-tenant).

---

## 2. Delivered changes

| Change | Delivered | Commits |
|---|---|---|
| p8-c001 project-store-reads | `ProjectStore::list()` + retargets + store dispatch | `c23a5cd` |
| p8-c002 archive-p6 | p6 → `specs/` | `c23a5cd` |
| (review fixes) | `parse_project_id` helper + 2 LOW cleanups | `3f7763a` |

All on `origin/main`.

---

## 3. Artifact Quality Summary

| Metric | Value |
|---|---|
| Changes with artifact-refiner QA | **0/2** (subsystem absent — all phases) |
| Changes with a dedicated **rust-reviewer** | **1/2** (c001 — the store-read code) |
| rust-reviewer verdict | **APPROVE** — 0 CRITICAL / 0 HIGH / 1 MED / 2 LOW (all fixed) |
| First-pass CI-gate pass rate | 2/2 |
| `cargo test` runs used (budget ≤3) | **1** — all pass first run |

**The reviewer confirmed the one property that mattered** — no-forge-call for the
store reads is structurally airtight (the `dispatch_forge` split cleanly excludes
`project.*` while keeping `forge.table.*`). The MED finding (`project_id` parse
duplicated a 3rd time → extracted `parse_project_id`) is a genuine DRY win the
review surfaced; both LOWs were dead-code/comment cleanups.

**Cleanest execute yet:** c001 built whole → one batched `cargo check` → clippy →
one `cargo test` (33/33), all green first pass. The p7 `every_store_catalog_kind_is_dispatched`
integrity test did exactly its job — it **forced** the `STORE_KINDS` update the moment
two kinds moved to `Store`, so the guard paid off one phase after being added.

---

## 4. Technical debt

**New:** none of substance. The read path is complete and store-backed; the reviewer's
findings were all resolved in-phase.

**Carried (the accumulating infra-gated set — now the salient debt):**
1. **Live 3-plane smoke** — never run; the p5 mock-boundary proof still stands in for
   real wire compatibility. Deferred since phase 6.
2. **Durable-store runtime proof** — the `PgProjectStore` (now with `list`) is
   compiled + unit-shaped but **never run against a real Postgres** (`#[ignore]`d
   container tests; Docker unavailable every phase since 6).
3. **Postgres TLS** (`NoTls` connector) and **per-operator RLS** (single-tenant
   store, no ownership column) — both noted, both deferred.

**Carried (older, non-infra):** MCP multi-server; fabric WS subscriptions; OpenDesign;
A2UI/React UI; Tauri; knowledge-base. And the 2 archival partials (`p2-c002`,
`p1-c004`) still active (their deferred work is exactly items above).

---

## 5. Lessons captured

- **A read path can silently query the wrong plane long after the write path moves.**
  `project.inspect`/`project.list` kept hitting forge (with `"<unspecified>"`,
  ignoring `project_id`) for three phases after the write side became store-backed.
  When you move a write path to a new home, audit the *matching reads* in the same
  breath — they don't error, they just read the wrong thing.
- **A dedicated `TargetPort` variant makes the read/write symmetry self-checking.**
  Because `project.*` reads and writes now both route through `Store`, the p7
  integrity test caught the read migration automatically. Structure that makes the
  compiler/tests enforce consistency beats structure that relies on memory.
- **Extract the parse helper at the third copy, not the second.** The reviewer was
  right to flag `parse_project_id` at copy #3 — two copies is arguably a coincidence,
  three is a pattern. Cheap to extract, and it centralizes the error message shape.
- **Meta-lesson: watch for "productive avoidance."** Three clean no-infra phases in a
  row is a signal, not just a streak — the unblocked work is getting done *because*
  it's unblocked, while the higher-value infra-gated work (live smoke, durable proof)
  waits. Worth surfacing to the operator rather than defaulting to a fourth.

---

## 6. Recommended Next Phase

The in-memory surface is now very complete (full artifact CRUD, secured, four planes).
The highest-value remaining work is **infra-gated and has been deferred four phases
running**. Two honest options — this is genuinely the operator's call:

**Option A — `live-smoke-and-durable-proof` (infra-gated, HIGHER value):** stand up
real forge (pgrx PG-18) + gate + fabric; run the end-to-end operator flow AND the
`fpa-store-pg --ignored` container tests (put/get/**list**/restart) against real
services. This finally discharges debt #1 + #2 — the two biggest carried items — and
converts "compiled + unit-tested" into "proven against reality." **Only viable if the
sibling stack / Docker can be brought up.**

**Option B — a no-infra increment** (e.g. richer `project.list` filtering, or `project.delete`
to round out CRUD, or MCP multi-server). Keeps momentum but adds to an already-strong
in-memory surface while the infra debt grows.

**Recommendation:** **Option A if the operator can bring up Docker/the stack** — the
infra debt is now the project's biggest risk (nothing has run against real siblings),
and four phases of deferral is enough. **If the stack still can't run here, Option B**
(`project-delete-and-crud-completion` — the smallest coherent no-infra step) and keep
A queued. I will ask.

---

## 7. Reflect handoff

All 3 in-scope goals MET; rust-reviewer APPROVE (1 MED + 2 LOW fixed); the read path is
now store-backed, completing the artifact CRUD (no forge round-trips for project data).
Cleanest execute yet (1 of 3 test-waits). No new debt. Corrective action & recommended
next phase: the salient carried debt is now **infra-gated** (live smoke + durable-store
runtime proof, deferred 4 phases) — recommend **`live-smoke-and-durable-proof`
(Option A)** IF the operator can stand up the sibling stack, else a small no-infra
CRUD-completion step (Option B) with A queued. This is an operator decision I will put
to them at next-phase.
