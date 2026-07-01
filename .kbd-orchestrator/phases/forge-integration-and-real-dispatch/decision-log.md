### 2026-07-01T03:56:58Z — Analyze: integration phase, minimal adopt
Mode: stack-specified. Tiers: cargo-search + sibling-inspection.
- HTTP client: reqwest (already present) — no new runtime dep.
- Test mock: ADOPT wiremock 0.6 (dev-dep) — matches flint-gate's reqwest-client
  test precedent; async/tokio-native. httpmock (fabric Ory) is the alternative.
- GraphQL body: build with serde_json (no graphql_client codegen).
- Credential threading + task store: internal BUILD, not adopt. Recommend
  AuthContext.bearer (Q2) + RwLock<HashMap> task store. Decide threading at spec.
Open for spec: Q2 threading shape, Q3 OpenAPI-vs-GraphQL for describe, Q4 MCP identity.

### 2026-07-01T04:01:20Z — Spec: 4 OpenSpec changes (read-only forge integration)
Backend: OpenSpec. ZeeSpec: n/a. Scope: READ-ONLY (assessment §5).
- p2-c001-credential-threading   thread gate bearer surfaces→runner→ports; MCP identity (Q4 at execute)
- p2-c002-forge-read-integration fpa-forge on /openapi.json + /graphql; wiremock tests (depends c001)
- p2-c003-catalog-input-validation per-kind schemas + InvalidInput (independent)
- p2-c004-task-store             in-memory RwLock<HashMap>; A2A status/cancel (independent)
DECISION: AuthContext.bearer (Q2) chosen over separate request-context — smallest change.
All 4 pass openspec validate. Deps: c001→c002; c003,c004 independent.

### 2026-07-01T04:03:16Z — Plan: ordered 4 changes
Order: c001 (root) → c002 (needs c001) → c003, c004 (independent). Linear for single-dev.
First to apply: p2-c001-credential-threading. Waypoint refreshed; plan_complete=true.
