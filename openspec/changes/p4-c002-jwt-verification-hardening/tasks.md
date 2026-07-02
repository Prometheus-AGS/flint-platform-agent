## 1. Trusted-header detection (trust path)

- [x] 1.1 Add config `FPA_TRUSTED_IDENTITY_HEADERS` (list of header names gate injects, e.g. `X-User-Id,X-Org-Id`).
- [x] 1.2 In `identity.rs`, when ALL configured trusted headers are present, build `OperatorContext` from them (trusted; no verify). No universal marker — deployment-configured.

## 2. JWKS verification (direct path)

- [x] 2.1 Add config `FPA_JWKS_URL` (the IdP JWKS gate uses); a JWKS client with a bounded cache (~300s) using reqwest.
- [x] 2.2 Verify direct tokens: `decode_header` → select JWK → `DecodingKey::from_jwk` → `Validation::new(alg)` → `decode`. Keep the HS256-secret path.
- [x] 2.3 Retire the unverified decode: a direct token with no usable key → `NoIdentity` (reject).

## 3. Verification

- [x] 3.1 `cargo check/clippy/fmt` green; never log tokens/claims.
- [x] 3.2 Test: request with configured trusted headers → trusted context (no verify).
- [x] 3.3 Test: direct token, valid signature → context; invalid → `NoIdentity`.
- [x] 3.4 Test: direct token, no key available AND no trusted headers → `NoIdentity` (not unverified accept).
- [x] 3.5 Test: JWKS fetched at most once within TTL (cache).
