## Why

`OryIdentityVerifier` uses RS256 exclusively — no HS256 path exists. The dev IdP that
lets the smoke mint a verifiable bearer must supply:
- An **RSA public key** at a JWKS URL the fabric gateway fetches.
- A matching **private key** the smoke uses to sign JWTs.

These are **throwaway smoke keys** (not real secrets, not production credentials), generated
once and committed to the repo like the existing `smoke-hs256-secret-not-a-real-credential`.
They are public-key-cryptographic but their exposure is deliberate (the fabric gateway only
needs the public key; the private key only lets you authenticate against the smoke's fake IdP).

## What Changes

- Generate a **2048-bit RSA key pair** (offline, no Docker). Store under `smoke/dev-idp/`:
  - `private-key.pem` — the smoke mints JWTs with this.
  - `jwks.json` — the public key in JWKS format, served at runtime. Fabric's
    `OryIdentityVerifier` fetches this URL, extracts the key by `kid`, verifies RS256.
- A `smoke/dev-idp/README` explains these are throwaway smoke artifacts.

## Capabilities

### New Capabilities
- `dev-idp-keys`: A static RSA key pair that lets the smoke mint RS256 JWTs verifiable by fabric's OryIdentityVerifier — the foundation for the realtime-receipt proof.

## Impact

- New directory `smoke/dev-idp/` (3 files). No code change. Not a secret (the private
  key is intentionally committed, same precedent as the HS256 secret in the composes).

## Open Questions
- **Key size:** 2048-bit is standard; 4096 adds no real benefit for a smoke-only key.
  Confirm 2048 is sufficient for `DecodingKey::from_jwk` (it is — RSA minimum).
