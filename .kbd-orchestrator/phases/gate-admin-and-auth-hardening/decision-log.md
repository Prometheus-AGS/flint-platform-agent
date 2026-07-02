### 2026-07-02T21:50:41Z — Analyze: minimal adopt (reuse existing deps)
- fpa-gate list_routes: BUILD thin reqwest (GET /v1/admin/routes). REJECT
  flint-gate-client git-dep — it's WS-heavy (tokio-tungstenite/bytes/futures) +
  cross-org (Know-Me-Tools); not worth it for one GET.
- JWKS verify: REUSE jsonwebtoken 9.3.1 jwk module (pub mod jwk confirmed; == gate's
  jsonwebtoken 9). Cached fetch mirrors gate's 300s TTL.
- Position-dependent trust: internal BUILD; Q1 (gate-injected-vs-direct detection)
  recommend a gate marker header (fails safe: no marker ⇒ verify). Decide at spec.
- Forge update/delete: reuse graphql_exec if in-scope; recommend DEFER.

### 2026-07-02T22:01:34Z — Sibling drift re-check (pre-spec, user-requested)
- forge: NEW REST CRUD (p3-c013/c014) mounted via reflection_router. DECISION (user):
  add forge-REST-sync change to this phase; migrate fpa-forge from pg_graphql to REST.
- gate: admin surface expanded (health/ready/cache + routes CRUD + API-key mgmt).
  DISCREPANCY: admin_app mounted w/o /v1/admin nest but flint-gate-client prepends it —
  confirm real path at execute, don't hardcode.
- fabric: no drift (/healthz unchanged); phase-3 fabric health still correct.

### 2026-07-02T22:11:37Z — Spec: 3 changes (deep gate re-review applied)
Deep-verified flint-gate source before finalizing:
- c001 gate list_routes: PATH RESOLVED = bare GET /routes on admin port (admin_router
  mounts bare, no /v1/admin nest; flint-gate-client's /v1/admin prefix is stale). New
  admin surface also has /api-keys + /signing-keys (out of scope).
- c002 JWT hardening: CORRECTED — no single gate marker; inject_headers are per-route
  config (X-User-* + minted JWT). Trust model = CONFIGURED trusted headers
  (FPA_TRUSTED_IDENTITY_HEADERS); verify direct tokens against the IdP JWKS
  (FPA_JWKS_URL = same IdP gate uses), not a gate endpoint.
- c003 forge-rest-sync (NEW, drift): migrate fpa-forge writes to forge REST CRUD.
All 3 pass openspec validate. Independent changes.
