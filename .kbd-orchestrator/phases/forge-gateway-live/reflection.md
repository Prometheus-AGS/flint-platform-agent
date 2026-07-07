# Reflection — forge-gateway-live

**Phase:** forge-gateway-live
**Stage:** reflect (phase-end)
**Date:** 2026-07-06
**Backend:** OpenSpec (3 changes, all `--strict` valid)
**Changes:** 3/3 complete

> Phase 11: pivoted from the blocked forge gateway goal to CI wiring once the
> assess re-verified that flint-forge#7 is still open. A clean, focused phase.

---

## Goal achievement

| # | Goal | Verdict | Evidence |
|---|------|---------|----------|
| 1 | Boot the real forge gateway (gated on flint-forge#7) | **NOT MET — correctly gated** | All 3 forge bugs still present on the live tree (confirmed in assess). The authored Dockerfile + bootstrap remain ready to flip via `--forge-full` when #7 lands. |
| 2 | Wire `run-real.sh` into CI | **MET** | Per-PR stub smoke job in ci.yml (6/6 green local dry-run); opt-in real-smoke workflow authored + inert (token-deferred). |
| 3 | Re-verify forge#7 first; pivot if still open | **MET** | Assessed the live tree first, found all 3 bugs open, correctly pivoted to goal 2. Decision log captures the 3 operator decisions. |

**Overall: 2/3 goals MET; goal 1 correctly NOT MET (gated as designed — the goal's own success criterion allowed this).**

---

## Delivered changes

- **c001 ci-stub-smoke** — `smoke` job in `.github/workflows/ci.yml`, `ubuntu-latest`, no secrets, no siblings. Invokes `smoke/run.sh` (the proven stub runner). **Local dry-run: 6/6 PASSED** (`smoke.spec.ts` explicit filter; no browser install). Also fixed `run.sh` to pass `smoke.spec.ts` explicitly (was `npm test` which matched both specs with the updated `testMatch`).
- **c002 real-smoke-workflow** — `.github/workflows/real-smoke.yml` authored + INERT (`workflow_dispatch`-only, `schedule:` commented, fail-fast without `SIBLING_CLONE_TOKEN`, `forge_full` input off by default, pinned sibling refs). Honestly ships un-run; enable by adding one secret.
- **c003 make-readme** — `Makefile` (`smoke`/`smoke-real`/`smoke-real-nobuild`/`smoke-real-forge`/`help`) + `smoke/README` fully rewritten (stub vs real, `--no-build` workflow, profiles, CI jobs, token enablement).

---

## Artifact Quality Summary

No artifact-refiner (no constraint file). QA = YAML lint (both workflows parse) + local dry-run.

| Metric | Value |
|--------|-------|
| Changes with a QA gate | 3/3 |
| YAML syntax valid | ci.yml, real-smoke.yml — both OK |
| Live-verified | c001 (stub smoke 6/6 local); c003 (`make help`/`make -n`) |
| Honestly-inert | c002 (can't run live without token; verified by inspection) |
| No agent Rust change | Rust CI unchanged; no cross-repo deps |

---

## Technical debt introduced

1. **`run.sh` was calling `npm test` (ambiguous)** — fixed in this phase (now `npx playwright test smoke.spec.ts`). This was a latent bug from the `testMatch` widening in p10-c005; caught here before CI ever ran both specs unexpectedly.
2. **Pinned sibling refs** in `real-smoke.yml` will need bumping when the proven-green commits advance. Documented as "bumpable" — a light ongoing maintenance task.
3. **Goal 1 (real forge gateway) still blocked** — flint-forge#7 open. The authored tooling (`fdb-gateway.Dockerfile`, bootstrap, `--forge-full` profile) is ready; the gate is forge's to open.

---

## Lessons captured

- **Assess-first pays off here too:** re-verifying forge#7 against the live tree before writing any spec confirmed the pivot was correct and prevented wasted spec/execute work on a still-blocked goal.
- **`testMatch` widening has a surface area:** widening `testMatch` to include `smoke.real.spec.ts` made `npm test` ambiguous — the stub runner would have picked up both specs. Caught by grounding `run.sh` before shipping. Always grep for `npm test` / `playwright test` calls when the test glob changes.
- **Inert workflows are honest artifacts:** `real-smoke.yml` ships without being runnable — that's the *correct* state given no token exists. Documenting the fail-fast + the enablement path (rather than hardcoding or omitting it) makes the token addition a one-step operation.

---

## Recommended next phase

**`realtime-receipt`** — the deferred phase-10 debt: prove that the agent actually receives a fabric `EventEnvelope` end-to-end (forge write → CDC → subscribe → agent's `/fabric/subscribe` SSE). Requires a **dev IdP** (a tiny JWKS the fabric gateway trusts via `GATEWAY_JWKS_URL`, plus a matching bearer the smoke mints) and a **Keto seed step** (write `subscribe`/`view` relation tuples for the subject/channel/envelope). The real-sibling stack is already in place; this is the missing auth piece.

Alternatively, if flint-forge#7 lands first, pivot to a **`forge-gateway-boot`** mini-phase (flip `--forge-full`, prove the agent's forge-read hops).
