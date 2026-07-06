# Decision Log — forge-gateway-live

### 2026-07-06 — Analyze: CI shape (no external stack discovery)
Mode: stack-specified (GitHub Actions). No library adopt — the stub smoke is already
self-contained + CI-runnable; the real smoke's constraints are infra, not tooling.

**Facts established (evidence):**
- No self-hosted runner (`gh api …/actions/runners` → total_count 0) → real smoke can't
  be a per-PR gate.
- gate + forge are cross-org (`Know-Me-Tools`); fabric + this repo are `Prometheus-AGS`.
  Cloning gate/forge in CI needs a cross-org PAT/App-token secret.
- Stub smoke (compose.smoke.yml) is self-contained (agent+postgres+wiremock, frf-domain
  vendored in-repo) → runnable per-PR with zero secrets.

**Decision (recommended, pending operator confirm at spec):**
- Per-PR CI guard = the STUB smoke (add a `smoke` job to ci.yml).
- REAL smoke = opt-in `workflow_dispatch` (+ optional nightly) that clones siblings @
  pinned refs with a cross-org token; NOT per-PR. If no token is provisioned, keep the
  real smoke local-only (a `make` target) until then.
- Provenance: research + repo facts. Awaiting operator confirmation of the token/runner
  question (Open Question 2).

### 2026-07-06 — Blocked: goal 1 (real forge gateway) — flint-forge#7 still open
All 3 forge bugs present on the live tree; issue OPEN. Gated (--forge-full off), out of
scope this phase. Ready to flip once forge fixes #7.

### 2026-07-06 — Spec: 3 operator decisions (AskUserQuestion)
- **CI shape:** stub smoke per-PR + real smoke as opt-in nightly/dispatch. Provenance: user.
- **Cross-org token:** DEFER — author real-smoke.yml inert (workflow_dispatch only, documented
  SIBLING_CLONE_TOKEN secret, no schedule). Provenance: user.
- **Realtime-receipt proof:** DEFER to a focused follow-up phase (not this one). Provenance: user.
Result: 3 changes — c001 ci-stub-smoke (per-PR), c002 real-smoke-workflow (inert, token-deferred),
c003 make-readme. G1 real forge gateway remains BLOCKED on flint-forge#7 (out of scope).
