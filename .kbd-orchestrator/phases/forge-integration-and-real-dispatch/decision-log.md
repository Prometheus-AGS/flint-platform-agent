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

### Execute: decisions resolved
- c001 MCP identity (Q4): MCP tools/call callers send the gate JWT as
  Authorization: Bearer on POST /mcp → reuse the OperatorContext extractor.
  Closes the MCP-identity debt fully.
- c003 validation: adopt jsonschema crate (full JSON Schema validation) —
  verify MSRV vs 1.93 before pinning (phase-1 lesson).

### c002 executed: forge read integration
Q3 resolved: describe_table uses OpenAPI component schemas (no GraphQL
introspection needed). list_tables ← GET /openapi.json (public). graphql_query
helper forwards bearer → RLS; missing bearer/401 → Unauthorized. 6 wiremock tests.
Live-forge smoke (4.5) NOT run — needs local Postgres-backed Quarry; deferred.

### c003 executed: input validation via jsonschema
Adopted jsonschema 0.46.8 (MSRV OK vs 1.93). CatalogEntry gains input_schema_json
(raw JSON text; const array can't hold Value) parsed by input_schema(). Runner
validates after permission, before dispatch → AppError::InvalidInput. Design catch:
null/absent input normalized to {} so no-arg calls pass an object schema while
required fields (project.inspect→project_id, forge.table.describe→name) enforce.
MCP tools/list surfaces real per-kind schemas. Tests: schema-parse, invalid-input
(no port call), valid-required-input.
