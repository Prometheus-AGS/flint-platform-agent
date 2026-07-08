## 1. Update the realtime test in smoke.real.spec.ts

- [x] 1.1 Add a const at the top of the file:
  - `const DEV_PRIVATE_KEY = fs.readFileSync(path.join(__dirname, 'dev-idp/private-key.pem'), 'utf8')`
  - `const KETO_WRITE = process.env.KETO_WRITE_URL ?? 'http://localhost:14467'`
  - `const FABRIC_TENANT_ID = '00000000-0000-0000-0000-000000000001'` (fixed CDC tenant)

- [x] 1.2 Helper `mintDevBearer()` — mint an RS256 JWT:
  ```ts
  jwt.sign(
    { sub: 'smoke-realtime', tenant_id: FABRIC_TENANT_ID,
      aud: 'frf-gateway', jti: randomUUID(), exp: Math.floor(Date.now()/1000)+3600 },
    DEV_PRIVATE_KEY,
    { algorithm: 'RS256', keyid: 'dev-smoke-key-1' }  // kid matches JWKS
  )
  ```

- [x] 1.3 Helper `seedKeto(namespace, object, relation, subject)` — `PUT` to
  `KETO_WRITE/relation-tuples` with `{namespace_id: namespace, object, relation, subject_id: subject}`.
  Verify the Keto v0.12 REST write body format (namespace_id vs namespace — check the API).

- [x] 1.4 Replace the existing realtime test body:
  - fixed `channelId = randomUUID()`, fixed `envelopeId = randomUUID()` (chosen once)
  - `mintDevBearer()`
  - `seedKeto("default", channelId, "subscribe", "smoke-realtime")`
  - `seedKeto("default", envelopeId, "view", "smoke-realtime")`
  - Open SSE: `fetch(${AGENT}/fabric/subscribe?channel=${channelId}, {headers: {Authorization: Bearer ${token}}})`
  - 500ms delay, then `POST ${FABRIC}/v1/publish` with the full EventEnvelope JSON
    (include `id: envelopeId`, `channel: {id: channelId, tenant_id: FABRIC_TENANT_ID, path: "entity/smoke/realtime"}`).
  - Read the SSE stream; assert it contains `envelopeId` within 15s.

## 2. Verification (integration milestone)

- [~] 2.1 `smoke/run-real.sh --no-build` against the c002-wired compose. **Result: 4 PASS
  + 1 SKIPPED.** The 4 real HTTP hops pass green (agent health, auth rejection, `fabric.health`
  through the real fabric gateway, project CRUD through real Postgres). The receipt test is
  marked `test.fixme` (reports SKIPPED) — it is **blocked on an upstream fabric bug**, not an
  agent defect: `frf-broker-iggy` subscribe/publish target different Iggy streams, so
  `consumer.init()` fails "Stream not found" and the agent bridge 502s. The agent side is
  proven correct (opens the WS, forwards the RS256 bearer; fabric verifies it and reaches
  `consumer.init` before erroring). Filed **Prometheus-AGS/flint-realtime-fabric#2**; see
  `smoke/KNOWN-ISSUES.md` (KI-1). Flip `test.fixme` → `test` for a true 5/5 once fabric ships
  the fix.
- [x] 2.2 Keto write body format: check the Keto v0.12 API (the provider sends `subject_id`;
  verify whether `namespace_id` or `namespace` is the correct field name for the v0.12
  REST API before coding). Confirm with a direct curl if unsure.
- [x] 2.3 If the test times out: check the agent logs for the `/fabric/subscribe` connection
  attempt; check the fabric-gateway logs for the JWKS fetch + JWT verification outcome.
