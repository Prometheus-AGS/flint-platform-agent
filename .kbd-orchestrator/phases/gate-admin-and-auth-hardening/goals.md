# Goals — gate-admin-and-auth-hardening

Seeded from `forge-writes-and-realtime/reflection.md` §6.

**Context verified (2026-07-02) against flint-gate source:**
- **Gate admin API is fully specified.** flint-gate ships a Rust client
  (`flint-gate-client`) over `/v1/admin/`: `GET /health`, `GET /routes`,
  `POST /routes` (upsert), `DELETE /routes/{id}`. NOTE: the admin prefix is
  `/v1/admin`, not bare `/routes` — the current `GateAdapter` stub guessed wrong.
- **Gate publishes JWKS.** `flint-gate-core` has `jwks_url` config + `jwt_verify.rs`
  (`JwkSet`, 300s cache). Gate signs with **HS256** (shared secret) OR
  **RS256/ES256** (PEM key). So full verification = HS256 via secret (already
  supported) + RS256/ES256 via fetched JWKS.

## Goals

- Implement `fpa-gate` against gate's admin API (`/v1/admin/routes`, `/health`):
  real `list_routes`, replacing the `Downstream` stub (closes gate half of debt #3/#5).
  Decide: reuse `flint-gate-client` (git-dep) vs a thin `reqwest` client.
- **Full JWT signature verification** against gate's JWKS: keep HS256-secret path,
  add RS256/ES256 via `jsonwebtoken`'s JWK support with a cached JWKS fetch
  (mirror gate's 300s cache). Absence of a verification key should mean **reject**,
  not "decode unverified" (retire the interim stopgap — carried debt #4).
- (Stretch) forge **update/delete** mutations to round out writes (debt #5).

## Deferred (still out of scope)

fabric WS subscriptions + AG-UI feed (own realtime phase), OpenDesign plugin,
React/Vite UI + generator, Tauri, knowledge base, durable task store.

## Open questions (resolve at assess)

- Reuse `flint-gate-client` crate (cross-org git-dep: Know-Me-Tools) vs thin
  reqwest client? Git-dep gives typed routes but adds a cross-org build dep.
- Does the agent verify JWKS itself, or is it acceptable that gate already verified
  upstream and the agent only *reads* claims? (Reflection assumes gate is the
  boundary — reconcile "full verification in the agent" against "gate already did
  it". May mean the agent verifies only when it independently receives a token.)
