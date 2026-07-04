## Why

The gate-identity extractor decodes JWTs **unverified** when `FPA_GATE_JWT_KEY` is
unset (interim stopgap, debt #4). That is insecure for a directly-received token.
Per the assessment's correction, verification is **position-dependent**: trust
gate-injected identity (the agent behind gate), verify only tokens received
directly. This change implements that model and retires the unverified decode.

**Source correction (2026-07-02):** gate does NOT set one fixed marker header. Its
`inject_headers` are **per-route config** (the example injects `X-User-Id`,
`X-User-Email`, `X-Org-Id`, and optionally **mints a fresh JWT** via `mint_jwt`).
And `jwks_url` is the **external IdP's** JWKS that *gate itself* fetches to verify
inbound tokens. So the model is:

- **Trust the gate-injected path via CONFIGURED trusted headers.** The operator
  declares which headers gate injects for this deployment (e.g. `X-User-Id`,
  `X-Org-Id`) via config (`FPA_TRUSTED_IDENTITY_HEADERS`). When those are present
  (the request came through gate), build identity from them — trusted, no verify.
  There is no universal gate marker; trust is deployment-configured.
- **Verify the direct path against the IdP JWKS.** When no trusted headers are
  present and a raw `Authorization` bearer is, verify it against a configured
  `FPA_JWKS_URL` (the same IdP JWKS gate uses): fetch `JwkSet` (reqwest, cached
  ~300s), `DecodingKey::from_jwk`, `Validation::new(alg)` — RS256/ES256; keep the
  HS256-secret path.
- **Retire the unverified decode.** A raw bearer with no usable verification key
  and no trusted headers is **rejected** (`NoIdentity`), never decoded unverified.
- Reuse `jsonwebtoken` 9.3.1's `jwk` module (already a dep) — no new crate. Never
  log tokens/claims.

## Capabilities

### New Capabilities
- `jwt-verification-hardening`: Position-dependent identity — trust gate-injected, verify direct tokens against gate's JWKS — replacing the interim unverified decode.

### Modified Capabilities

## Impact

- `fpa-gateway` (`identity.rs`: trusted-header detection, JWKS fetch+cache, verify path);
  config gains `FPA_TRUSTED_IDENTITY_HEADERS` (list) + `FPA_JWKS_URL` (IdP JWKS).
- No new deps (`jsonwebtoken` `jwk` + `reqwest` present).
- Independent of the other p4 changes.

## Open Questions
- **Q1 RESOLVED (source):** trust is via CONFIGURED trusted headers, not a fixed
  gate marker — gate's `inject_headers` are per-route config (`X-User-*` + minted
  JWT). Agent trusts a configured header set; else verifies. Fails safe.
- **Q3:** `FPA_JWKS_URL` = the external IdP JWKS (same one gate uses). Confirm
  runtime reachability + which IdP (Kratos/Hydra) at execute.
