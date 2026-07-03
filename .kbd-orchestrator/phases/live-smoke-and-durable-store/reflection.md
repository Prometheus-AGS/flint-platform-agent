# Reflection — live-smoke-and-durable-store

**Phase:** live-smoke-and-durable-store (re-scoped at assess)
**Date:** 2026-07-03
**Changes:** 2/2 executed (p6-c001, c002), all CI-green and pushed
(`e7726a9`, `6c2bee1`).

> Sycophancy gate applied. The honest headline: this phase **shipped the durable
> store's code but did not prove it against a real Postgres** (Docker unavailable),
> and the phase's *original* headline goal — the live 3-plane smoke — was **deferred
> at assess**, not delivered. What's here is real and reviewed; what's unproven is
> named as unproven.

---

## 1. Goal achievement (against the RE-SCOPED goals)

| Goal (re-scoped) | Verdict | Evidence |
|---|---|---|
| Durable `ProjectStore` (agent-owned Postgres) | **MET (code) / UNPROVEN (runtime)** | c001: new `fpa-store-pg` crate over tokio-postgres+deadpool; JSONB store; config-selected. Unit round-trip passes. The **restart-survival proof is `#[ignore]`d** (Docker down) — the durability is not runtime-verified this session. |
| Archive p5 OpenSpec changes | **MET** | c002: p5-c001..c004 archived; 4 capabilities now in `openspec/specs/`; no Rust touched by the archival. |

**Against the ORIGINAL goals.md, honestly:**
| Original goal | Verdict |
|---|---|
| Live 3-plane smoke | **NOT MET — deferred** (operator decision at assess; Docker unreachable, forge has no compose). |

**Overall:** the re-scoped work is 100% code-complete and reviewed; but this phase
did **not** close the mock-boundary gap p5 left open — it moved the durable-store
half forward and pushed the live proof to a later phase. Net progress on debt is
real but partial.

---

## 2. Delivered changes

| Change | Delivered | Commits |
|---|---|---|
| p6-c001-durable-project-store | `fpa-store-pg` crate + async `AppState` store-selection + config | `e7726a9`, `6c2bee1` (review fixes) |
| p6-c002-archive-p5-changes | p5-c001..c004 → `specs/` | `e7726a9` |

Pushed to `origin/main`.

---

## 3. Artifact Quality Summary

| Metric | Value |
|---|---|
| Changes with artifact-refiner QA | **0/2** (subsystem absent — all phases) |
| Changes with a dedicated **rust-reviewer** | **1/2** (c001 — the new infra adapter) |
| rust-reviewer findings (c001): CRITICAL/HIGH/MED/LOW | 0 / 0 / 1 / 1 (both fixed) |
| First-pass CI-gate pass rate | 2/2 |
| `cargo test` runs used (budget ≤3) | **0 heavy** — fast unit suite only |

**The rust-reviewer confirmed the parts that matter and caught two real
improvements.** Clean: hexagonal boundary (adapter imported only by the composition
root, never `fpa-app`), error mapping (all → `PortError`, no lib `unwrap`), secret
safety (URL never logged, redacted in Debug), SQL correctness, and the container
test genuinely proving restart-survival. Fixed: a MEDIUM (manual config field-copy
silently dropped `sslmode`/multi-host → replaced with `cfg.url` so deadpool parses
the whole URL — more correct *and* less code) and a LOW (unused `thiserror` dep).

**Implementation-first held on a smaller change:** c001 built whole → one batched
`cargo check` (55s cold for the new PG deps) → clippy → fast tests. **Zero heavy
`cargo test` waits** — the JSONB round-trip + schema-fit unit tests validated every
DB-independent path; the DB-dependent proof is the (deferred) container test.

---

## 4. Technical debt

**New / carried (honest):**
1. **Durability is not runtime-proven.** The `#[ignore]`d testcontainers restart
   test is the proof and it did **not run** (Docker unreachable). The store is
   compiled + unit-tested, not verified against a real Postgres. Run
   `cargo test -p fpa-store-pg -- --ignored` once Docker is up.
2. **Live 3-plane smoke still deferred** — the p5 mock-boundary gap is still open;
   this phase did not close it. It needs its own phase + a runnable sibling stack.
3. **No TLS to Postgres.** The connector is `NoTls`; a remote DB needs a rustls
   connector (`tokio-postgres-rustls` or deadpool TLS). The `cfg.url` fix stops
   silently dropping `sslmode` intent, but TLS itself is not wired. Local/trusted
   network only for now.
4. **Only `project.create` persists.** `application.define` still has no store/forge
   home; richer `project.create` input (nested applications/sub-agents/schemas) is
   not yet accepted.
5. **Earlier-phase archival backlog** — p1–p4 OpenSpec changes are still in the
   active `changes/` list (only p5 was archived this phase). Housekeeping.

**Carried (older):** MCP multi-server; fabric WS subscriptions; OpenDesign;
A2UI/React UI; Tauri; knowledge-base.

---

## 5. Lessons captured

- **Assess must test *runnability*, not just read code.** The single most valuable
  act this phase was probing the Docker daemon + sibling compose *before* committing
  to the live smoke — it turned a doomed goal into a clean operator re-scope. When a
  goal depends on external infra, verify the infra can run in the actual environment
  first. (Reinforces [[live-smoke-deferred-durable-store-agent-pg]].)
- **`#[ignore]` + a documented run command is the honest way to ship a DB adapter
  without a DB.** The code is real and reviewed; the proof is present but gated on
  infra. The run output says `ignored, requires Docker` — no silent green.
- **A new infra adapter goes in its own crate, wired only at the composition root.**
  `fpa-store-pg` is imported by `fpa-gateway` alone; `fpa-app` stays infra-free. The
  reviewer confirmed the boundary at the Cargo-dependency level — that's the check
  that matters for hexagonal.
- **Let the pool library parse the whole URL.** Manually copying config fields drops
  `sslmode`/`options`/multi-host silently. `cfg.url = Some(...)` is both safer and
  shorter — the reviewer's fix improved correctness while removing code.
- **Making a constructor async ripples to its callers — check them first.**
  `AppState::new` became async for the Postgres connect; both callers (`main`, the
  integration test) were already in async contexts, so the ripple was clean. Verify
  that before flipping a widely-called signature.

---

## 6. Recommended Next Phase

Two viable directions; recommend the operator choose based on whether the stack can
be stood up.

**Option A (infra-gated) — `live-smoke-real-stack`:** the deferred goal. Stand up
real forge (pgrx PG-18) + gate + fabric compose; run the p5 operator flow *and* the
`fpa-store-pg --ignored` container test against real services; fix wire drift. This
finally closes debt #1 + #2 + runtime-proves the durable store. **Prereq:** colima/
Docker up + the forge image built — confirm feasibility first (it blocked this phase).

**Option B (no-infra) — `project-artifact-depth`:** richer `project.create` input
(accept + validate nested applications/sub-agents/schemas), an `application.define`
persistence home, and clear the p1–p4 archival backlog. All implementable without
Docker — keeps momentum if the stack still can't run.

**Recommendation:** **Option B** unless the operator can bring the stack up now —
it's unblocked here and there is real product depth to add, whereas Option A is
still gated on the exact infra that blocked this phase. Revisit A the moment Docker
is available.

---

## 7. Reflect handoff

Re-scoped goals MET in code (durable `fpa-store-pg` adapter — reviewed, no
CRITICAL/HIGH, 2 findings fixed; p5 archived), but **durability is not
runtime-proven** (restart test `#[ignore]`d, Docker down) and the **live 3-plane
smoke stayed deferred** — this phase did not close the p5 mock-boundary gap.
Corrective action & recommended next phase: **`project-artifact-depth`** (Option B,
no-infra: richer project.create, application.define home, p1–p4 archival) unless the
operator can stand up the sibling stack now, in which case **`live-smoke-real-stack`**
(Option A) to run everything against real services and finally close the runtime
proof. Open: TLS-to-Postgres (rustls connector) is a smaller residual either way.
