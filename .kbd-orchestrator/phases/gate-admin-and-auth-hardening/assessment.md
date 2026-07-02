# Assessment — gate-admin-and-auth-hardening

**Phase:** gate-admin-and-auth-hardening
**Date:** 2026-07-02
**Method:** Inspection of this repo + flint-gate (`flint-gate-client`, `jwt_verify.rs`,
`middleware/pipeline.rs`) + flint-forge (`fdb-auth`).

---

## 0. Bottom line + a correction to the seed

Both goals are unblocked. **But the reflection's framing of "full JWT verification
in the agent" needs correcting** against the verified auth model:

- **gate mints a fresh JWT and injects it** (`Authorization: Bearer` + `X-User-*`
  headers) to its upstreams (`pipeline.rs` `inject_headers` / `mint_jwt`). So when
  the agent sits **behind gate**, it receives **gate-injected identity it can
  trust** — no JWKS verification needed there (this is exactly the "gate is the
  boundary" invariant).
- **forge independently verifies** the bearer it receives
  (`fdb-auth::rls_from_bearer` → `forge_identity::verify_and_build` — real verify).
  So a service that receives a **raw** token verifies it.

**Reconciliation (the design decision this phase must make):** the agent's
verification posture is **position-dependent**:
1. **Behind gate** (normal): trust gate-injected identity; do not re-verify.
2. **Direct token** (e.g. an MCP host presenting a JWT NOT via gate): the agent
   **should verify** against gate's JWKS, like forge does — replacing the interim
   "decode unverified" stopgap **for that path only**.

So debt #4 is closed not by "always verify" but by "**verify when we independently
receive a token; trust when gate injected it**." This keeps the gate-only-boundary
invariant intact while removing the insecure unverified decode.

---

## 1. Verified contracts

### Gate admin API
- flint-gate ships **`flint-gate-client`** (v0.1.0, org Know-Me-Tools) over
  `/v1/admin/`: `GET /health`, `GET /routes`, `POST /routes` (upsert),
  `DELETE /routes/{id}`. Client uses `bearer_auth`; maps 401/403 → auth error.
- **Correction to current stub:** `GateAdapter::list_routes` targets bare
  `/routes` — the real path is `/v1/admin/routes`. Must fix.

### Gate JWKS verification (the pattern to mirror for the direct-token path)
- `flint-gate-core::auth::jwt_verify`: fetches `JwkSet` from `jwks_url` (reqwest,
  300s cache), `decode_header` → select JWK → `DecodingKey::from_jwk` →
  `Validation::new(header.alg)` → `decode::<Claims>`. Supports HS256 (secret) +
  RS256/ES256 (JWK). This is exactly what the agent's direct-token path needs.

---

## 2. Current state of this agent (baseline)

| Component | State | Gap |
|---|---|---|
| `fpa-gate::list_routes` | `PortError::Downstream("not implemented")`; `GateAdapter` holds `admin_url` only | Implement against `/v1/admin/routes`; add reqwest client + bearer; fix the path |
| Gate admin task (`application.deploy`) | routes to `gate.list_routes` (stub) | Real admin call; deploy = route upsert (`POST /v1/admin/routes`) — but that's a **write**; scope carefully |
| JWT verify (`identity.rs`) | HS256-secret when `FPA_GATE_JWT_KEY` set; **decode-unverified otherwise** (interim) | Add JWKS path (RS256/ES256) for direct tokens; make "no key + not gate-injected" **reject**, not unverified |
| Gate-injected identity (`X-User-*`) | not read; only `Authorization` JWT decoded | Add a trusted path: when identity arrives via gate headers, trust it |

---

## 3. Gap analysis per goal

Legend: ✅ ready · 🟡 partial · ❌ to build

| Goal | Status | Gap |
|---|---|---|
| `fpa-gate` admin `list_routes` (real) | ❌ | reqwest `GET /v1/admin/routes` with bearer; map 401/403→Unauthorized, others→Downstream, unreachable→Transport. **Decide: reuse `flint-gate-client` git-dep vs thin reqwest.** |
| Full JWT verification (JWKS) | 🟡 | Add JWKS fetch+cache (mirror gate's 300s) + RS256/ES256 via `DecodingKey::from_jwk`; keep HS256-secret. **Reject** unverified when no key AND not gate-injected. |
| Trust gate-injected identity | ❌ (new, from the correction) | Read `X-User-*` / gate-minted `Authorization` as trusted when it arrives via gate; only the direct path verifies. |
| Forge update/delete (stretch) | ❌ | `graphql_exec` mutation for `update…Collection`/`deleteFrom…Collection`; defer unless time. |

---

## 4. Key design decisions (for analyze/spec)

1. **Position-dependent verification (the crux).** Define how the agent knows it's
   "behind gate" vs "direct": likely a gate-set marker header (e.g. `X-Gate-Verified`)
   or trusting `X-User-*` presence. If present → trust; else → verify JWKS. **This is
   the central spec decision; get it right or the auth model is wrong.**
2. **Reuse `flint-gate-client` vs thin reqwest.** Client gives typed routes + the
   correct `/v1/admin` prefix for free, but adds a **cross-org git-dep**
   (Know-Me-Tools) on every build machine. Thin reqwest keeps deps local. Recommend
   thin reqwest for `list_routes` (small surface); revisit if admin surface grows.
3. **`application.deploy` is a gate write** (`POST /v1/admin/routes`). Keep this
   phase to **read** (`list_routes`) + verification; treat deploy as a follow-on
   (mirrors the read-before-write discipline from forge).

---

## 5. Recommendations / watch items

- **Correct the reflection's over-broad goal in the spec:** verify on the direct
  path, trust on the gate path — do not "always verify" (would contradict the
  gate-only-boundary invariant and duplicate gate's work).
- **Reuse jsonwebtoken's JWK support** (already a dep) — no new crate for JWKS;
  add a small cached fetch. `reqwest` present.
- **Never log tokens/claims** (extend existing redaction to the JWKS path).
- **Keep gate admin to reads this phase**; deploy/route-write is a follow-on.
- Still **no artifact-refiner QA** — CI + wiremock/smoke remain enforcement.

---

## 6. Open questions for analyze/plan

- **Q1 (crux):** how does the agent detect gate-injected (trusted) vs direct
  (must-verify) requests? Gate marker header, `X-User-*` presence, or config?
- **Q2:** reuse `flint-gate-client` (cross-org git-dep) vs thin reqwest for admin?
- **Q3:** is gate's `jwks_url` a well-known, reachable endpoint the agent can fetch
  at runtime, and what config surfaces it (`FPA_GATE_JWKS_URL`)?
- **Q4:** forge update/delete in-scope or deferred? (Recommend defer.)

---

## 7. Stage handoff

Unblocked; gate ships a typed admin client + JWKS-verify pattern to mirror. **Key
correction:** verification is position-dependent (verify direct tokens, trust
gate-injected) — not "always verify" as the seed implied; this closes debt #4
without breaking the gate-only-boundary invariant. Core work = real `fpa-gate`
`list_routes` (fix `/v1/admin` path) + JWKS verify for the direct path + trust the
gate-injected path. Open for analyze: the trusted-vs-verify detection (Q1),
client-reuse (Q2), JWKS config (Q3), forge update/delete scope (Q4).
