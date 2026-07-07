## Why

With the dev IdP (c001) + the wired compose (c002), the full realtime-receipt path is
unblocked. The current realtime test in `smoke.real.spec.ts` asserts only the **auth
boundary** (fabric correctly rejects the no-IdP smoke). This change replaces that test
with the **happy-path receipt proof**: the agent receives a real `EventEnvelope` through
the full chain — subscribe → publish → SSE bridge emits it.

## What Changes

Update the `realtime` test in `smoke.real.spec.ts`:
1. **Mint a dev RS256 JWT** (`jsonwebtoken` 9.x + the dev private key PEM, claims: `sub`,
   `tenant_id`, `aud="frf-gateway"`, `exp`, `jti`).
2. **Seed 2 Keto tuples** via `PUT http://localhost:14467/relation-tuples`:
   - `{namespace: "default", object: channel_id, relation: "subscribe", subject_id: sub}`
   - `{namespace: "default", object: envelope_id, relation: "view", subject_id: sub}`
   The `envelope_id` is a **fixed UUID chosen at the top of the test** (the smoke controls
   the publish body, so the `id` field can be set deterministically).
3. **Subscribe** to `channel_id` via the agent's `/fabric/subscribe?channel=...` SSE
   bridge, using the dev bearer.
4. **Publish** to fabric `POST /v1/publish` with the full `EventEnvelope` JSON (including
   the fixed `id`). Auth is bypassed (`DEV_NO_AUTH=true`) on the publish path.
5. **Assert** the SSE bridge emits the envelope containing the fixed `envelope_id` within
   a timeout.

The existing 4 HTTP hops (healthz, auth-reject, fabric.health, project CRUD) are unchanged.
The auth-boundary test is **replaced** by the receipt test (it was a stepping-stone probe;
the receipt test is the real proof).

## Capabilities

### New Capabilities
- `realtime-receipt`: The agent provably receives a fabric EventEnvelope end-to-end — forge-style write → /v1/publish → CDC/subscribe path → agent's /fabric/subscribe SSE bridge emits the frame.

## Impact

- `smoke/smoke.real.spec.ts` (realtime test update). No agent Rust change.
  `smoke/run-real.sh` unchanged (the spec still runs under `smoke.real.spec.ts`).

## Open Questions
- **`tenant_id` value in the JWT:** must match `RelationTuple.tenant_id` in both Keto
  tuples. Use a fixed deterministic UUID (e.g. the CDC tenant from p10-c003:
  `00000000-0000-0000-0000-000000000001`) so it's consistent across the test.
- **Keto write body format:** confirmed from frf-authz-keto provider.rs — the smoke
  sends `{namespace, object, relation, subject_id}`. Verify the Keto v0.12 REST API
  accepts this body shape (it's the Ory Keto v0.12 write API — `PUT /relation-tuples`).
