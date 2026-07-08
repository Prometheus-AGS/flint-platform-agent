# smoke/dev-idp — Throwaway RSA key pair for the realtime-receipt smoke

## What these files are

These are **deliberately committed, smoke-only RSA artifacts** — not real secrets.

| File | Purpose |
|---|---|
| `private-key.pem` | Signs RS256 JWTs in `smoke.real.spec.ts` (JWT mint step). |
| `public-key.pem` | The matching public key (derived from private; for reference). |
| `jwks.json` | The public key in JWKS format, served by the `dev-idp-jwks` compose service. Fabric's `OryIdentityVerifier` fetches this URL and verifies the RS256 signature. `kid: "dev-smoke-key-1"`. |

## Why they are committed

Committing the private key is intentional and safe here:
- This key only authenticates **against this smoke's dev IdP** — the fabric gateway
  trusts it only when `GATEWAY_JWKS_URL` points at the dev JWKS server (the real
  compose stacks use a real gate JWKS).
- Exposure lets the smoke run without any setup step. Same precedent as the
  `smoke-hs256-secret-not-a-real-credential` HS256 key in the compose files.
- Rotate by running `openssl genrsa … > private-key.pem` + re-generating `jwks.json`
  (see `scripts/` or the keygen commands in the c001 tasks). No production impact.

## Do NOT use these keys for anything other than the local smoke.
