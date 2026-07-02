# Analysis — gate-admin-and-auth-hardening

**Phase:** gate-admin-and-auth-hardening
**Date:** 2026-07-02
**Mode:** stack-specified
**Inputs:** `assessment.md` (which corrected the seed's auth goal).

> Minimal-adopt phase. The assessment already researched the contracts from
> source; the only build-vs-adopt calls are the gate-client-reuse question and
> confirming `jsonwebtoken`'s JWK support. Both resolve to **reuse existing deps**.

## 1. Landscape summary

- **JWKS verification reuses what we have.** `jsonwebtoken` **9.3.1** (our exact
  workspace pin, == gate's `jsonwebtoken = "9"`) exposes `pub mod jwk`
  (`JwkSet`, `DecodingKey::from_jwk`) — the precise API gate's `jwt_verify.rs`
  uses. No new crate: add a cached JWKS fetch (reqwest, 300s TTL like gate) +
  RS256/ES256 verify, keep the HS256-secret path.
- **The gate admin client is NOT worth adopting.** `flint-gate-client` 0.1.0
  would give typed routes + the correct `/v1/admin` prefix, but its deps include
  **`tokio-tungstenite` (WS), `bytes`, `futures`** — it's a full streaming client,
  not a thin admin helper. As a **cross-org git-dep** (Know-Me-Tools) it drags a
  WS stack + cross-org SSH onto every build/CI machine for a single `GET`.
  **Thin reqwest wins** (`GET /v1/admin/routes` + bearer + 401/403 mapping).
- **Everything else is internal design** — chiefly the position-dependent trust
  model from the assessment (trust gate-injected, verify direct).

## 2. Build-vs-adopt calls

| Gap | Verdict | Confidence |
|---|---|---|
| `fpa-gate` list_routes | **BUILD** thin reqwest (`/v1/admin/routes`); reject gate-client git-dep | high |
| JWKS verify (direct path) | **REUSE** `jsonwebtoken` 9.3.1 `jwk` (no new crate) | high |
| Position-dependent trust | **BUILD** (internal; Q1 detection at spec) | medium |
| Forge update/delete (stretch) | reuse `graphql_exec`; recommend **defer** | high |

## 3. The decision that matters (for spec)

**Q1 — how the agent detects gate-injected (trusted) vs direct (must-verify).**
This is the auth model's hinge (from the assessment correction). Options:
- **(A) Gate-set marker header** (e.g. `X-Gate-Verified: 1` or the presence of
  gate's `X-User-*` set) → trust; otherwise verify the `Authorization` JWT against
  JWKS. Simple, explicit.
- **(B) Config/deployment mode** (agent knows if it's deployed behind gate) →
  coarser, less flexible per-request.

Recommend **(A)** with a specific, documented gate marker — per-request, explicit,
and fails safe (no marker ⇒ verify). **Decide the exact header at spec**, ideally
matching whatever gate actually sets in `pipeline.rs` `inject_headers`.

## 4. Evidence (tiered)

- **Tier 3 (dep inspection):** `jsonwebtoken` 9.3.1 has `pub mod jwk` (our pin);
  `flint-gate-client` 0.1.0 deps include `tokio-tungstenite`/`bytes`/`futures`
  (WS-heavy). Both first-hand from installed source / sibling Cargo.toml.
- **Tier 3 (sibling parity):** gate + this agent both on `jsonwebtoken` 9 →
  verify-shape parity. gate admin path `/v1/admin/*` confirmed via
  `flint-gate-client`.
- **Tiers 1/2/4:** not needed.

## 5. Open questions (carried to spec)

1. **Q1 (crux):** exact gate-injected-vs-direct detection — recommend a gate
   marker header (option A); confirm what gate sets.
2. **Q2:** RESOLVED — thin reqwest, not the gate-client git-dep.
3. **Q3:** `jwks_url` config surface for the agent (`FPA_GATE_JWKS_URL`) + is the
   endpoint reachable at runtime? Confirm at spec/execute.
4. **Q4:** forge update/delete — recommend **defer** (keep phase focused on gate +
   auth).

## 6. Handoff to Spec

Adopt set is empty (all reuse): thin reqwest for `fpa-gate`, `jsonwebtoken` 9.3.1
`jwk` for JWKS. Spec must decide **Q1** (the trusted-vs-verify detection — the auth
hinge) and **Q3** (JWKS config), keep gate admin to **reads** (`list_routes`), and
**defer** forge update/delete + route-writes.
