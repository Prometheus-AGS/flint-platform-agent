# Plan — realtime-receipt

**Phase:** realtime-receipt
**Stage:** plan
**Date:** 2026-07-06
**Backend:** OpenSpec (`openspec/changes/p12-c00*`), 3 changes.
**Analyze skipped** — grounding was done in assess (live fabric code read, both unknowns
resolved). No external library research needed; `jsonwebtoken` 9.0.2 already present.

## Grounding summary (carried from assess)

- **Dev IdP**: RS256-only (hardcoded in OryIdentityVerifier). Static RSA key pair +
  JWKS JSON file served in the compose. No issuer check when `JWT_ISSUER` env absent.
  Required JWT claims: `sub`, `tenant_id` (UUID), `aud="frf-gateway"`, `exp`.
- **Keto**: 2 exact tuple seeds with deterministic envelope `id` — smoke chooses a
  fixed UUID before publish, seeds `(subject, subscribe, channel)` and
  `(subject, view, envelope_id)` via `PUT :4467/relation-tuples` before subscribing.
  Keto write port `:4467` not yet exposed to host — needs a port mapping.
- **No agent Rust change.** Pure smoke composition + the realtime spec update.

## Change ordering (dependency-driven)

| Order | Change | Why here | Touches | Risk |
|---|---|---|---|---|
| 1 | **p12-c001-dev-idp-keys** | Foundation: generate the RSA key pair + JWKS file that everything else references. No live dependency; fully offline. | `smoke/dev-idp/` (new) | Low (keygen script + static JSON) |
| 2 | **p12-c002-compose-dev-idp** | Wires the dev IdP into the fabric compose stack: adds the JWKS server service, updates `GATEWAY_JWKS_URL`, adds Keto `:4467` port mapping. Must exist before the spec can run. | `smoke/compose.fabric.yml`, `smoke/compose.real.yml` | Low-med (compose edit; validate each file) |
| 3 | **p12-c003-realtime-receipt-spec** | The proof itself: updates `smoke.real.spec.ts` to mint a dev-IdP RS256 bearer, seed the 2 Keto tuples, subscribe, publish with the known envelope `id`, assert the SSE bridge emits it. **This is the integration milestone.** | `smoke/smoke.real.spec.ts` | Med (live smoke run needed to prove it; the one full-run wait) |

## Verification strategy

- **c001**: offline — keygen + JWKS JSON format validation (a `python3 -c` json.loads or
  `jq .`). No Docker.
- **c002**: `docker compose -f smoke/compose.fabric.yml config` (validate) + a bare
  `up --no-build` to confirm the JWKS server starts and serves the file.
- **c003**: a live `smoke/run-real.sh --no-build` run. The realtime-receipt test should
  pass (event received via SSE bridge); the HTTP hops should still pass. This is the
  ≤3 `cargo test`-equivalent wait (the full live smoke run for the phase).

## Success criteria

1. The fabric gateway starts with the dev JWKS URL and verifies bearers signed with the
   dev private key (`OryIdentityVerifier` lookup succeeds).
2. The Keto seed step writes 2 tuples (subscribe + view) for the smoke's fixed envelope id.
3. `smoke.real.spec.ts` realtime test passes: the SSE bridge receives the `EventEnvelope`
   with the expected id within the timeout. The existing 4 HTTP hops still pass.

## First change to apply

**p12-c001-dev-idp-keys** — generate the RSA key pair + write the static JWKS JSON.
Fully offline; can be verified with `openssl` or `python3`.
