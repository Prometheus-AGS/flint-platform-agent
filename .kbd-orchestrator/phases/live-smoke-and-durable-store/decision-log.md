# Decision Log — live-smoke-and-durable-store

### 2026-07-03 — Live smoke scope (assess)
Decision: DEFER the live 3-plane smoke to a future phase. Provenance: **user**
(AskUserQuestion). Reason: Docker daemon unreachable here; forge has no compose (heavy
custom pgrx PG-18 image, no gateway orchestration). p5 mock-boundary proof stands.

### 2026-07-03 — Durable store location (assess/implicit)
Decision: agent-owned Postgres (new fpa-store-pg crate), NOT a forge flint_meta.projects
table. Provenance: user (scope) + implicit (the forge-table option inherits the deferred
forge/Docker blocker). Consistent with the p5 persistence decision.

### 2026-07-03 — Postgres driver (analyze)
Decision: tokio-postgres 0.7.18 + deadpool-postgres 0.14.1 (Tier 3 verified). sqlx 0.9.0
rejected (compile-time-check friction; different driver family than the sibling).
Provenance: research + sibling-consistency (fabric uses tokio-postgres 0.7).

### 2026-07-03 — Test infra (analyze)
Decision: testcontainers 0.27.3 + testcontainers-modules 0.15.0 (dev-dep) for a
restart-survival Postgres test, #[ignore]d-by-default (Docker down). Unit round-trip
tests run always. Provenance: research.
