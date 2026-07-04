# Reflection — project-artifact-depth

**Phase:** project-artifact-depth
**Date:** 2026-07-04
**Changes:** 3/3 executed (p7-c001, c002, c003), all CI-green and pushed
(`391f2ca`, `1c2e046`).

> Sycophancy gate applied. Unlike phase 6, this phase closed with **no
> deferred-runtime caveat** — everything shipped is unit-proven against the
> in-memory store, no infra was needed. The one honest note is the 2 archival
> partials that were deliberately NOT force-archived.

---

## 1. Goal achievement

| Goal | Verdict | Evidence |
|---|---|---|
| Richer `project.create` (nested aggregate, validated) | **MET** | c001: accepts applications/sub_agents/schemas/realtime/entity_meta; serde-typed validation; malformed nested item rejected before any write; server owns id + schema_version. 6 new unit tests. |
| `application.define` persistence home | **MET** | c002: store-backed — load project → immutable upsert `ApplicationDef` by id → put; unknown `project_id` rejected. |
| Archive p1–p4 backlog | **MET (12/14) + 2 skip-noted** | c003: 12 complete changes archived (16 capabilities now baselined); the 2 partials left active by design (below). |

**Overall: 100% of in-scope goals MET.** Both design leans from assess held
(friendly-map input; require-exists + upsert; `TargetPort::Store` routing).

---

## 2. Delivered changes

| Change | Delivered | Commits |
|---|---|---|
| p7-c001 richer-project-create | nested serde validation + `TargetPort::Store` | `391f2ca` |
| p7-c002 application-define-home | store-backed upsert; `ApplicationDef` re-export | `391f2ca` |
| p7-c003 archive-p1-p4 | 12 archived; 2 partials skip-noted | `391f2ca` |
| (review fixes) | `Store` integrity test + test cleanup | `1c2e046` |

All on `origin/main`.

---

## 3. Artifact Quality Summary

| Metric | Value |
|---|---|
| Changes with artifact-refiner QA | **0/3** (subsystem absent — all phases) |
| Changes with a dedicated **rust-reviewer** | **1/3** (c001+c002 — the store-dispatch code) |
| rust-reviewer verdict | **APPROVE** — 0 CRITICAL / 0 HIGH / 0 MED / 2 LOW (both fixed) |
| First-pass CI-gate pass rate | 3/3 |
| `cargo test` runs used (budget ≤3) | **1** — all pass first run |

**The reviewer confirmed the two properties that mattered most** and both were
solid on the first implementation: **validation-before-write** holds in both
handlers (no partial-write path — every fallible step precedes the single `put`),
and **`schema_version` is fully server-owned** (the code maps only named nested
fields, never `schema_version`, so a client value is silently ignored — Base Rule
39). The 2 LOWs were test hygiene (a `Store` catalog-integrity guard mirroring the
Gate one; an unnecessary test-helper param) — both taken.

**Implementation-first, clean run:** c001+c002 (same files) built whole → one batched
`cargo check` (caught one missing `serde` dep) → clippy → fast unit tests. 1 of 3
test-waits used; the whole phase runs against the in-memory store (no Docker).

---

## 4. Technical debt

**New / carried (honest):**
1. **2 p1–p4 partials not archived** — `p2-c002` task 4.5 (optional manual live-forge
   smoke) and `p1-c004` task 3.2 (MCP multi-server, explicit carry-forward). Both are
   genuinely-deferred work, so per the no-force-archive rule they stay **active +
   noted**, not fake-completed. They archive when their deferred work lands (live
   smoke phase / MCP multi-server phase).
2. **Nested input is a light JSON-Schema guard + serde** — the catalog schema only
   checks `type: array`/`object` at the top; per-item validation is serde. That's the
   deliberate design (DRY, no per-field schema drift), but it means the *published*
   `input_schema` for `project.create` under-describes the real accepted shape. A
   richer published schema (schemars-generated from `Project`) is a later nicety.
3. **`realtime` inline deser** — handled separately from the array fields (it's an
   object, not a `Vec`); minor asymmetry, fine.

**Carried (older, still true & still deferred):** live 3-plane smoke; durable-store
runtime proof; Postgres TLS; MCP multi-server; fabric WS subscriptions; OpenDesign;
A2UI/React UI; Tauri; knowledge-base. And **p6 itself is now archivable** (`✓
Complete`) — a trivial housekeeping follow-up.

---

## 5. Lessons captured

- **When the domain types already derive `Deserialize`, serde IS the validator.**
  The richer-input goal needed no per-field JSON Schema — `serde_json::from_value`
  into the typed nested `Vec`s rejects malformed shapes precisely, and stays DRY
  against `#[non_exhaustive]` domain types. Reach for a hand-authored schema only for
  the *coarse* guard (required top-level keys).
- **Server-own version/identity fields; never let the client set them.** Building
  from `Project::new` and mapping only named nested fields (never `schema_version`)
  is the clean way to enforce Base Rule 39 — and a one-line test locks it in.
- **A dedicated `TargetPort` variant beats "intercept-before-the-other-port".** The
  earlier "`project.create` is Forge-targeted but handled before the forge call" was
  muddy; `TargetPort::Store` makes agent-owned aggregate writes first-class and the
  dispatch match self-documenting (and `#[non_exhaustive]` makes a missing arm a
  compile error).
- **Add the integrity test when you add the dispatch family.** The reviewer's LOW —
  a `Store` counterpart to the Gate integrity test — is the same "no silent
  catch-all" discipline; adding it with the variant (not after) is cheaper.
- **Inspect a "partial" before archiving — it may be honestly-deferred, not
  unfinished.** Both p1–p4 partials were deliberate carry-forwards, not overlooked
  tasks. Reading the trailing task line turned a "reconcile" into a correct
  "skip + note".

---

## 6. Recommended Next Phase

The agent's artifact + persistence layers are now solid (typed nested Project, store
home, durable adapter). Two credible directions:

**Option A — `project-read-and-list` (no-infra, recommended):** the write path is
rich but the **read path is thin** — `project.inspect`/`project.list` are catalogued
`TargetPort::Forge` and hit forge (which has no projects table). Retarget them to
`TargetPort::Store`: `project.inspect` → `store.get`; `project.list` → a new
`ProjectStore::list()` port method (+ in-mem + Pg impls). This makes the store a
complete CRUD surface and is fully testable without Docker — the natural completion
of the artifact work.

**Option B — `live-smoke-real-stack` (infra-gated):** still the deferred goal; only
viable if the sibling stack can be stood up (Docker + forge image).

**Recommendation: Option A** — it's unblocked, completes the read side of the
artifact CRUD the last three phases built the write side of, and clears a real
inconsistency (`project.inspect`/`list` currently query forge for data that lives in
the agent's store). Also fold in the trivial **p6 archival** (housekeeping).

**Open questions (for A's assess):** does `ProjectStore::list()` need pagination/
filtering now or is a full list fine at this scale (lean: full list now, paginate
later); and should `project.inspect` return the whole aggregate or a summary (lean:
whole aggregate — it's the read of what `project.create` wrote).

---

## 7. Reflect handoff

All 3 in-scope goals MET; rust-reviewer APPROVE (0 CRITICAL/HIGH, 2 LOW fixed) with
validation-before-write and server-owned `schema_version` both confirmed. Clean
implementation-first run (1 of 3 test-waits). Honest note: 2 p1–p4 archival partials
were deliberately skip-noted (deferred work, not unfinished), not force-archived.
Corrective action & recommended next phase: **`project-read-and-list`** (Option A,
no-infra) — retarget `project.inspect`/`project.list` to the store (+ a
`ProjectStore::list()` port method), completing the artifact CRUD read side that the
write side (p5–p7) has outpaced; fold in the trivial p6 archival. Live smoke stays
deferred (infra-gated). Open: list pagination now-or-later, inspect full-vs-summary.
