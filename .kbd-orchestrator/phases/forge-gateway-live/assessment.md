# Assessment — forge-gateway-live

**Phase:** forge-gateway-live
**Stage:** assess
**Date:** 2026-07-06
**Docker:** healthy (colima vz, 6 CPU / 12 GiB)

> Operator directive: **consider the latest changes in `../flint-forge`.** Goal 3 makes
> this phase's shape contingent on **flint-forge#7** — so the re-check IS the first job.

---

## 1. Goals recap (from goals.md)

1. **Boot the real forge Quarry gateway** (`fdb-gateway`) once flint-forge#7 is fixed —
   the authored Dockerfile + roles/`flint_meta` bootstrap are ready; flip `--forge-full`.
2. **Wire `run-real.sh` into CI** (the deferred no-infra Option B).
3. **Re-verify flint-forge#7 first**; if still open, keep the gateway gated and pivot to
   CI wiring + (optionally) a real realtime-receipt proof via a dev IdP.

---

## 2. flint-forge#7 re-check (the gating dependency) — VERDICT: STILL BLOCKED

Checked the **live** forge tree (committed HEAD `8c7955c` — unchanged since phase 10; tree
still actively dirty, mid-`p3-c019`). **All three bugs are still present**, and the GitHub
issue is **OPEN**:

| forge#7 bug | Status on live tree | Evidence |
|---|---|---|
| 1. pgvector/pg_graphql PG18 pins | **NOT fixed** | `docker/postgres/Dockerfile`: `PGVECTOR_REF=v0.8.0`, `PG_GRAPHQL_REF=v1.5.11` (unchanged) |
| 2. duplicate migration versions | **NOT fixed** | `migrations/`: two `0005_`, two `0006_` still present (a new `0009_` was added; the collisions remain; the files are still `??` untracked/in-flight) |
| 3. `.dockerignore` excludes `images/` | **NOT fixed** | `.dockerignore` line 22: `images/` (still excludes what the pgrx Dockerfile COPYs) |

→ **Goal 1 remains BLOCKED.** Nothing in the agent repo can fix these (they're forge's,
in forge's actively-dirty tree — another session's work). Per goal 3's decision tree:
**keep the forge gateway gated (`--forge-full` off) and pivot this phase to goals 2 + a
realtime-receipt proof.** Do NOT wait on forge#7.

**Small positive delta:** forge added `0009_flint_kiln_cedar_policies.sql` — but this does
NOT resolve the `0005`/`0006` collisions the gateway's `sqlx::migrate!` chokes on.

---

## 3. Goal 2 (CI wiring) — the real constraint is architectural

**Existing CI:** `.github/workflows/ci.yml` has 4 jobs — `fmt`, `clippy` (pedantic,
`-D warnings`), `test` (`cargo test --workspace`), `msrv` (1.93). All run on stock
`ubuntu-latest` with `checkout@v4` of **this repo only**.

**Confirmed CI is NOT broken by phase 10:** the c004 cross-repo `frf-domain` path dep was
**vendored** into `vendor/frf-domain` in c005 — no cross-repo path deps remain, so
checkout-only CI still resolves everything. The `#[ignore]` Docker tests are skipped by
`cargo test` default. **So the existing Rust CI is green as-is.**

**The gap — and why "wire run-real.sh into CI" is not straightforward:** `run-real.sh`
needs (a) a Docker daemon AND (b) the sibling repos (`../flint-gate`,
`../flint-realtime-fabric`, `../flint-forge`) as build contexts. A stock GH Actions runner
has Docker but **NOT the siblings** — they don't live in this repo. So the *real* smoke
cannot run in stock CI. Options (for analyze/spec):
- **(a) Self-hosted / composite runner** that checks out the siblings alongside this repo,
  then `run-real.sh --no-build` (needs the images pre-built or a big build budget).
- **(b) Scheduled/manual workflow** (`workflow_dispatch` / nightly) that `git clone`s the
  siblings at pinned refs, builds once, runs the smoke. Heavy; not per-PR.
- **(c) CI runs the STUB smoke (p9 wiremock path — self-contained), keep the REAL smoke a
  local/`make` target.** The stub smoke needs no siblings; it's the honest per-PR guard.
  Lean: **(c) for per-PR + (b) as an opt-in nightly** — matches "the stub is the
  fast/reliable path; real is high-fidelity."

**Also missing:** no `make`/`just`/task runner (goal 2 mentions "GH Actions vs a make
target"); `smoke/README` documents `run.sh` but not `run-real.sh` / `--no-build`.

---

## 4. Optional: real realtime-RECEIPT proof (phase-10 debt)

Phase 10's SSE bridge reaches real fabric but fabric's Ory-JWKS + per-event Keto `view`
correctly reject the smoke (no real IdP). Proving *receipt* needs: a JWKS the fabric
gateway trusts (`GATEWAY_JWKS_URL`) + a matching bearer + seeded Keto `subscribe`/`view`
tuples for the subject/channel/envelope. A **dev IdP** (a tiny JWKS the smoke mints
against) + a Keto seed step could close it — but it's real work and couples to fabric's
authz internals. Candidate for THIS phase only if goal-2 is light; else defer.

---

## 5. Gap summary

| # | Gap | Size | Blocker? |
|---|---|---|---|
| G1 | Real forge gateway boot | — | **BLOCKED on forge#7 (all 3 bugs open)** — gated, out of scope this phase |
| G2 | Per-PR CI guard for the smoke | Med | No — but the REAL smoke can't run in stock CI (no siblings). Lean: wire the STUB smoke per-PR |
| G3 | Nightly/opt-in REAL smoke in CI | Med-high | No — needs siblings cloned + Docker + build budget (self-hosted or scheduled) |
| G4 | `make`/task target + `smoke/README` for `run-real.sh --no-build` | Small | No |
| G5 | Realtime-receipt proof via a dev IdP + Keto seed | Med-high | No — optional; couples to fabric authz |

---

## 6. Open questions (for analyze/operator)

1. **CI shape for the smoke:** confirm the lean — **stub smoke per-PR (self-contained) +
   REAL smoke as an opt-in nightly/dispatch** (siblings cloned at pinned refs)? Or is a
   self-hosted runner with the siblings available (then per-PR real smoke is viable)?
2. **Sibling refs in CI:** if CI clones siblings, pin them to specific commits (forge/gate/
   fabric are all actively dirty) — or vendor more? (frf-domain is already vendored.)
3. **Realtime-receipt this phase or defer?** It's the most interesting agent-capability
   gap but the heaviest; include only if goal-2 lands light.
4. **forge#7 nudge:** should we help land the forge fixes in a separate session (operator
   owns forge), or purely wait? (Assessment recommends: keep gated, don't block.)

---

## 7. Handoff to analyze/plan

**flint-forge#7 is STILL OPEN — all 3 bugs present on forge's live (dirty) tree; goal 1
(real forge gateway boot) stays BLOCKED/gated.** Pivot the phase to **CI wiring**: the
existing Rust CI is green (frf-domain vendored, no cross-repo deps), but the REAL smoke
can't run in stock CI (siblings aren't in-repo). Recommend **stub smoke per-PR +
opt-in nightly real smoke**, plus a `make`/README target for `run-real.sh --no-build`.
Optional stretch: a real realtime-receipt proof via a dev IdP + Keto seed. Key decision
for analyze: the CI shape (per-PR stub + nightly real vs self-hosted per-PR real).
