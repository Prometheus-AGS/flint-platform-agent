# Reflection — real-sibling-smoke

**Phase:** real-sibling-smoke
**Stage:** reflect (phase-end)
**Date:** 2026-07-06
**Backend:** OpenSpec (6 changes, all `openspec validate --strict` clean)
**Changes:** 6/6 complete

> Capstone of the live-smoke arc: phase 9 proved the real agent against **stub**
> siblings (6/6); this phase swapped the stub for **real forge / gate / fabric** and
> proved actual wire compatibility — the last unknown. Operator directive: **nothing but
> REAL**.

---

## Goal achievement

| # | Goal | Verdict | Evidence |
|---|------|---------|----------|
| 1 | Real gate + fabric in the smoke (reachable now) | **MET (exceeded)** | c001 gate + c003 fabric each live-green standalone; c005 unified `smoke.real.spec.ts` **5/5** against real gate + fabric + agent Postgres. EXCEEDED: also built the fabric subscribe client (c004) + a `/fabric/subscribe` SSE bridge — originally a "later phase." |
| 2 | Real forge (STRETCH — pgrx build) | **PARTIAL (honestly scoped, as the goal permitted)** | c002: real forge CI PG-18 image + roles/flint_meta bootstrap + authored `fdb-gateway` Dockerfile — all PROVEN (PG healthy, bootstrap exits 0, gateway image builds). Live gateway **boot** blocked on a forge bug (duplicate migration versions). c006: full pgrx image blocked on a *different* forge bug (`.dockerignore` excludes what its Dockerfile COPYs). Both documented + reported (flint-forge#7). |
| 3 | `compose.real.yml` + `run-real.sh`; stub path stays | **MET** | `smoke/compose.real.yml` (8-service default + `forge-full` profile) + `run-real.sh` (serial build / `--no-build` fast path, `down -v` trap). Stub `run.sh` untouched. |

**Overall: ~90% — all primary goals MET, the STRETCH forge goal PARTIAL exactly as the
goal's own success criterion allowed ("document non-convergence honestly").**

## Delivered changes

- **c004 fabric-realtime-client** — `FabricClient::subscribe(channel, bearer) →
  BoxStream<EventEnvelope>` over `/ws/v1/subscribe`. Unit-proven (real WS handshake →
  `EventEnvelope` decode, 6/6). Reuses `frf-domain::EventEnvelope` (the real wire type —
  corrected from an early wrong assumption of `ContentBlock`).
- **c001 real-gate** — real flint-gate + postgres:16; agent `GET /routes` hop → 200 live.
- **c003 real-fabric** — real gateway + iggy + keto + postgres; agent `/healthz` → 200 live.
- **c005 compose-real-and-smoke** — unified stack + `/fabric/subscribe` SSE bridge +
  `frf-domain` vendored (portability fix); **smoke.real.spec.ts 5/5 green**.
- **c002 real-forge** — CI PG image + bootstrap + authored gateway Dockerfile (proven);
  gateway boot deferred (forge#7).
- **c006 full-pgrx-image** — compose authored + validated; build blocked upstream (forge#7).

## Artifact Quality Summary

No artifact-refiner (Rust repo, no constraint file wired). QA gate = `cargo check` +
`cargo clippy --workspace -D warnings` + `cargo fmt --check` + **live smoke**.

| Metric | Value |
|--------|-------|
| Changes with a code/QA gate | 6/6 |
| Rust changes clippy/fmt clean | c004, c005 (the code changes) — both green |
| Live-verified changes | c001, c003, c005 (green); c002 (PG plane green) |
| `cargo test` full-runs spent | 1 (c004 `fpa-fabric` unit) — well under the ≤3 budget |
| Live smoke result | `smoke.real.spec.ts` **5/5** |

## Technical debt introduced

1. **`frf-domain` vendored** (`vendor/frf-domain`) — pragmatic bridge; must re-sync if
   fabric's `EventEnvelope` schema changes. Real fix = a published `frf-domain` crate.
2. **Realtime event-RECEIPT not proven end-to-end** — the SSE bridge reaches real fabric
   but fabric's Ory-JWKS + per-event Keto `view` correctly reject the smoke (no real IdP).
   Proving receipt needs a real Ory Hydra/JWKS + seeded Keto tuples (deliberately out of
   scope — the agent must not fake the auth boundary).
3. **No read-only gate task kind** in the catalog — the unified smoke can't drive gate
   via A2A (only writes exist, which refuse). Gate proven standalone instead.
4. **Unified live run is `--no-build`-dependent** — the 12 GiB VM runs the 8-service
   stack fine but concurrent *builds* OOM it. Documented workflow: build once per-service,
   then `run-real.sh --no-build`.

## Lessons captured (→ knowledge base / memory)

- **Ground every sibling against its LIVE tree before authoring** — this caught the
  `ContentBlock`-vs-`EventEnvelope` wire type, gate's loopback-admin fail-safe, fabric's
  keto-config-file requirement, and 3 forge Dockerfile bugs. The discipline paid off
  repeatedly. (memories: fabric-subscribe-wire-contract, gate-admin-auth-smoke-posture,
  fabric-smoke-compose-grounding, forge-ci-pg18-image-two-bugs.)
- **Read-only consumption of sibling repos, no writes** — all three siblings were
  actively dirty (other sessions). Held the line: everything under `smoke/` + a vendored
  crate; nothing written into forge/gate/fabric. (memory: forge-bootstrap-owned-by-smoke.)
- **VM: run vs build** — the capacity "wall" was two fixable issues (a one-shot tripping
  `up --wait`; a testMatch typo) masking a working stack. The VM runs the stack fine;
  only concurrent builds OOM it. (memory: smoke-real-stack-run-vs-build.)
- **Process miss:** relaunched runs before confirming prior ones died → 3 overlapping
  builds crashed dockerd. Corrected — confirm-then-launch.

## Value shipped back to forge (unplanned but real)

Because *we* had Docker and forge was Docker-blocked, we surfaced + reported **3 forge
containerization bugs** on **Know-Me-Tools/flint-forge#7**: (1) pgvector/pg_graphql PG18
pins, (2) duplicate migration versions (breaks `sqlx::migrate!`), (3) `.dockerignore`
excludes what the pgrx Dockerfile COPYs. Each with a repro + suggested fix.

## Recommended next phase

**`forge-gateway-live` (small, gated on flint-forge#7)** — once forge fixes the dup
migration versions + `.dockerignore`, flip `--forge-full` on: the authored `fdb-gateway`
Dockerfile + bootstrap are ready, so the forge gateway should boot with near-zero smoke
changes, giving full-fidelity forge in the smoke. Pairs well with **wiring the real
smoke into CI** (the deferred no-infra Option B) so `run-real.sh` guards regressions.

Alternatively, if forge#7 lingers, pivot to **the next agent capability** (e.g. real
realtime event-receipt via a dev IdP, or MCP multi-server) — the real-sibling harness is
now in place to validate it.
