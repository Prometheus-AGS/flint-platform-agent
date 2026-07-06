# Analysis — forge-gateway-live

**Phase:** forge-gateway-live
**Stage:** analyze
**Date:** 2026-07-06
**Mode:** stack-specified (GitHub Actions is the CI stack — not a discovery). No
build-vs-adopt *library* choice; the "stack" is GH Actions + the already-present
self-contained stub smoke. Research was targeted at CI *shape*, not library selection.

> Assessment verdict carried in: **flint-forge#7 STILL OPEN** — goal 1 (real forge
> gateway boot) stays BLOCKED/gated. This phase = CI wiring (goal 2) + optional
> realtime-receipt stretch.

---

## Landscape (what already exists — nothing to adopt externally)

- **Existing CI** (`.github/workflows/ci.yml`): `fmt`, `clippy -D warnings`, `test`,
  `msrv 1.93` — all `ubuntu-latest`, checkout-only. **Green as-is** (frf-domain vendored
  in c005 → no cross-repo deps; `#[ignore]` Docker tests skipped).
- **Stub smoke** (`smoke/compose.smoke.yml` + `run.sh` + `smoke.spec.ts`): agent (built
  from THIS repo, `vendor/frf-domain` in-repo) + `postgres:17` + `wiremock`. **Fully
  self-contained — no sibling repos, no cross-org access.** This is the CI-runnable one.
- **Real smoke** (`compose.real.yml` + `run-real.sh` + `smoke.real.spec.ts`): needs
  Docker + the three sibling repos as build contexts.

## Evidence-backed findings

### Finding 1 — the real smoke CANNOT be a per-PR CI gate

- **No self-hosted runner** exists (`gh api …/actions/runners` → `total_count: 0`). So
  there is no runner with the siblings on disk; per-PR real smoke is not available.
- On a stock `ubuntu-latest` runner, the real smoke would have to **`git clone` all three
  siblings** each run and **build gate + fabric + agent from source** — minutes of heavy
  Rust/node builds per PR. Even locally this stack is at the 12 GiB ceiling (phase-10:
  concurrent builds OOM; only `--no-build` is reliable). Per-PR is infeasible.
- **Cross-org access:** this repo is `Prometheus-AGS/flint-platform-agent`. `fabric` is
  same-org (`Prometheus-AGS`), but **`gate` + `forge` are `Know-Me-Tools`** (cross-org,
  private). Cloning them in CI needs a **cross-org PAT / GitHub App token** as a secret —
  a real cost + a secret-management decision.

→ **Per-PR CI = the STUB smoke** (self-contained, fast, zero secrets). The **REAL smoke =
opt-in `workflow_dispatch` + optional nightly `schedule`**, clones siblings at pinned
refs, uses a cross-org token, `--no-build` where images can be cached. This matches the
house line: "the stub is the fast/reliable path; real is high-fidelity."

### Finding 2 — compose-in-CI needs no new tooling (ADOPT the stock runner)

`ubuntu-latest` ships Docker + Compose v2. A compose stack runs via a plain shell step
(`docker compose -f smoke/compose.smoke.yml up ...`). GH Actions `services:` only does
single containers, not a compose graph — so a shell step is the right shape. **No action
to adopt; no new dependency.** (The `run.sh`/`run-real.sh` runners already encapsulate
up→wait→smoke→down.)

### Finding 3 — the `make`/README target is trivial build-work

No `make`/`just`/task runner exists. A tiny `Makefile` (or documented commands) exposing
`smoke`, `smoke-real`, `smoke-real-nobuild` + a `smoke/README` update for
`run-real.sh --no-build` — small, no research.

### Finding 4 — realtime-receipt (optional stretch) is genuine build-work, no library

Proving fabric event RECEIPT needs a **dev IdP** (a tiny JWKS the fabric gateway trusts
via `GATEWAY_JWKS_URL` + a matching bearer) **and a Keto seed step** (write
`subscribe`/`view` relation tuples for the subject/channel/envelope). Both are
authored-in-smoke, not adopted. Heaviest item; include only if goal-2 lands light. No
external candidate.

---

## Build-vs-adopt calls

| Gap | Verdict | Why |
|---|---|---|
| Per-PR CI smoke | **BUILD (reuse stub)** — add a `smoke` job to ci.yml running the self-contained stub | No new deps; stub already CI-runnable |
| Real smoke in CI | **BUILD (dispatch/nightly workflow)** — clone siblings @ pinned refs + cross-org token | No self-hosted runner; cross-org secret needed → cannot be per-PR |
| compose-in-CI mechanism | **ADOPT stock** — `docker compose` on `ubuntu-latest` via a shell step | Built into the runner; `services:` can't do a compose graph |
| make/README target | **BUILD (trivial)** | No task runner exists |
| realtime-receipt proof | **BUILD (optional stretch)** — dev IdP JWKS + Keto seed | Couples to fabric authz; heaviest; defer unless goal-2 is light |

## Open questions (for spec/operator)

1. **CI shape — confirm:** per-PR **stub** smoke (self-contained, recommended) + **real**
   smoke as `workflow_dispatch`/nightly (siblings cloned, cross-org PAT). Agree? Or is a
   self-hosted runner going to be provisioned (then per-PR real becomes viable)?
2. **Cross-org token:** the real-smoke workflow needs a secret to clone `Know-Me-Tools/
   {flint-gate,flint-forge}` from `Prometheus-AGS/flint-platform-agent`. Provide a PAT/App
   token secret, or keep the real smoke **local-only** (a `make` target, no CI) until a
   self-hosted runner exists? (Lean: local-only real smoke first; CI stub now; add the
   nightly real workflow when a token/runner is available.)
3. **Realtime-receipt this phase?** Include the dev-IdP + Keto-seed stretch, or defer to a
   focused follow-up? (Lean: defer unless goal-2 is a single small change.)

## Budget

Tiers 1–3 only, internal/repo evidence + `gh api` (runners) + known GH Actions behavior.
No Tier-4 web research needed — the decision is a CI-shape call bounded by facts already in
hand (no self-hosted runner; cross-org siblings; stub is self-contained). Confidence: high.
