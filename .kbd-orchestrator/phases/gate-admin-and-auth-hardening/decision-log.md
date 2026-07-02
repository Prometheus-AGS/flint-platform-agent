### 2026-07-02T21:50:41Z — Analyze: minimal adopt (reuse existing deps)
- fpa-gate list_routes: BUILD thin reqwest (GET /v1/admin/routes). REJECT
  flint-gate-client git-dep — it's WS-heavy (tokio-tungstenite/bytes/futures) +
  cross-org (Know-Me-Tools); not worth it for one GET.
- JWKS verify: REUSE jsonwebtoken 9.3.1 jwk module (pub mod jwk confirmed; == gate's
  jsonwebtoken 9). Cached fetch mirrors gate's 300s TTL.
- Position-dependent trust: internal BUILD; Q1 (gate-injected-vs-direct detection)
  recommend a gate marker header (fails safe: no marker ⇒ verify). Decide at spec.
- Forge update/delete: reuse graphql_exec if in-scope; recommend DEFER.
