# Execution — gate-admin-and-auth-hardening

**Backend:** openspec. **Changes:** 0/3.

## Dispatch order (risk-ordered, all independent)
1. p4-c002-jwt-verification-hardening   ← first (auth; security-reviewed)
2. p4-c003-forge-rest-sync
3. p4-c001-gate-admin-list-routes

Per change: implement → ./scripts/ci-check.sh green → wiremock/smoke → mark DONE → next.

## Pre-flight (no blockers)
- All deps present (jsonwebtoken jwk 9.3.1, reqwest, wiremock). No cross-org git-deps.
- All external paths/URLs config-parameterized (verified from source):
  c002 FPA_TRUSTED_IDENTITY_HEADERS + FPA_JWKS_URL; c003 FPA_FORGE_REST_PREFIX;
  c001 gate admin base URL (path bare /routes).
- c002 requires a security-reviewer pass before commit.

## First pending change
p4-c002-jwt-verification-hardening.
