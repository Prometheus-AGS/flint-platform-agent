### 2026-07-01T03:56:58Z — Analyze: integration phase, minimal adopt
Mode: stack-specified. Tiers: cargo-search + sibling-inspection.
- HTTP client: reqwest (already present) — no new runtime dep.
- Test mock: ADOPT wiremock 0.6 (dev-dep) — matches flint-gate's reqwest-client
  test precedent; async/tokio-native. httpmock (fabric Ory) is the alternative.
- GraphQL body: build with serde_json (no graphql_client codegen).
- Credential threading + task store: internal BUILD, not adopt. Recommend
  AuthContext.bearer (Q2) + RwLock<HashMap> task store. Decide threading at spec.
Open for spec: Q2 threading shape, Q3 OpenAPI-vs-GraphQL for describe, Q4 MCP identity.
