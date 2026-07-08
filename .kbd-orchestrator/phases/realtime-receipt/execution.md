# Execution — realtime-receipt

**Stage:** execute
**Backend:** `openspec` (changes under `openspec/changes/p12-c00*`).
**Date:** 2026-07-07

## Backend selection

OpenSpec, all 3 changes `--strict` valid. No agent Rust change — pure smoke
composition + a TypeScript spec update. Implementation-first: build all 3, then verify.

## Dispatch order (from plan.md)

1. **c001** dev-idp-keys — RSA 2048 key pair generated (`openssl genrsa`), JWKS JSON
   produced (Python + `cryptography` library), JWT mint round-trip verified. Offline.
2. **c002** compose-dev-idp — JWKS server (`nginx:alpine`) + Keto `:4467` port mapping
   added to both `compose.fabric.yml` and `compose.real.yml`. Both validate OK.
3. **c003** realtime-receipt-spec — `smoke.real.spec.ts` updated: RS256 bearer mint,
   2 Keto tuple seeds (`PUT :14467/relation-tuples`, body `{namespace, object, relation,
   subject_id}` — confirmed from frf-authz-keto source), subscribe via agent SSE bridge,
   publish with deterministic `envelopeId`, assert receipt.

## Verification (the integration milestone)

`smoke/run-real.sh --no-build` — expects 5/5 PASS including the new receipt test.
This is the one full-run wait for the phase.
