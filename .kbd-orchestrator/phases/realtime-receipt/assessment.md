# Assessment — realtime-receipt

**Phase:** realtime-receipt
**Stage:** assess
**Date:** 2026-07-06
**Prior evidence:** phase-10 proved the SSE bridge reaches fabric's auth boundary; phase-11
wired CI. This phase proves the agent actually **receives** a fabric `EventEnvelope`.

---

## 1. Goals recap (from goals.md)

1. Prove event receipt end-to-end — forge write → CDC → `/ws/v1/subscribe` → agent's
   `/fabric/subscribe` SSE emits the `EventEnvelope`.
2. Smoke-owned dev IdP — a JWKS the fabric gateway trusts via `GATEWAY_JWKS_URL`.
3. Keto wildcard/subject-set vs post-publish seed — ground before speccing.

---

## 2. Q1: OryIdentityVerifier — exact requirements for a dev IdP (GROUNDED)

**Algorithm:** RS256 only (`Validation::new(Algorithm::RS256)` in verifier.rs:77). The
dev IdP must issue **RS256 JWTs** (asymmetric — no HS256 option in the verifier). A 2048-
bit RSA key pair, a JWKS endpoint serving the public key, and a valid JWT signed with the
private key satisfy the verifier.

**Audience:** required, must match `JWT_AUDIENCE` env. The real smoke composes set
`JWT_AUDIENCE = "frf-gateway"`. The dev bearer must carry `"aud": "frf-gateway"`.

**Issuer:** the gateway is wired with `OryIdentityVerifier::new(...)` (no issuer — see
main.rs:128-129). Issuer validation is **skipped**; only audience + signature matter.
(If `config.jwt_issuer` is `Some`, it switches to `with_issuer` which would also check
`iss` — but the current compose has no `JWT_ISSUER` env set, so `jwt_issuer` is `None`.)

**Claims needed in the JWT** (from `FrfClaims`):
- `sub` — becomes `claims.subject` (used for Keto tuples and subscribe authz check)
- `aud` — must include `"frf-gateway"`
- `exp` — standard expiry
- `tenant_id` (optional in the struct but used in Keto tuples as `RelationTuple.tenant_id`)
- `session_id` (used as `consumer_id` in the subscribe pipeline — should be set to avoid a
  UUID parse panic if the code expects it to be a valid UUID)

→ **Verdict for the spec:** The dev IdP is a **smoke-owned RSA key pair + a tiny JWKS
endpoint + a script/step that mints RS256 JWTs**. Options:
- **(a) Static: pre-generated RSA key pair** — private key baked into the smoke (not a
  real secret, throwaway), public key served as a static `jwks.json` file from `nginx` or
  a `python3 -m http.server` one-shot in the compose.
- **(b) Dynamic: `jose` / `python-jose` / a tiny Node script** mints a fresh JWT each run.

Both are smoke-owned (nothing into the fabric repo). Option (a) is simpler and avoids an
extra service; the JWKS file is a static artifact. Lean: **(a) static RSA + nginx JWKS**.

---

## 3. Q2: Keto view check — exact approach (GROUNDED)

**The per-event check** (subscribe.rs:90-100): every delivered envelope is filtered by
`(subject, view, envelope.id)`. `envelope.id` is the `EventId` UUID.

**No wildcard in the current namespace config.** The smoke's `keto.yml` has only a bare
`default` namespace (no OPL `permits`/`related` rules). Subject-set expansion is not
configured. Exact tuples only.

**The key insight: the smoke controls the envelope `id`.** `POST /v1/publish` accepts a
full `EventEnvelope` JSON — including the `id` field. So the smoke can:
1. **Choose a deterministic `id`** (a fixed UUID) for the envelope it will publish.
2. **Seed two Keto tuples before publishing** (via `PUT :4467/relation-tuples`):
   - `(subject, "subscribe", channel_id)` — grants the subscribe access check.
   - `(subject, "view", envelope_id)` — grants the per-event filter.
3. **Subscribe** to the channel.
4. **Publish** the envelope with that exact `id`.
5. **Assert** the SSE bridge emits the envelope.

Both tuples are seeded with known values before either the subscribe or the publish call.
The timing constraint is eliminated. The `RelationTupleBody` shape (from the Keto provider
code): `{namespace: "default", object: "<uuid>", relation: "<subscribe|view>", subject_id: "<subject>"}`.

**Keto write endpoint:** `PUT :4467/relation-tuples` (Keto write port, NOT read :4466).
The smoke calls this HTTP endpoint directly — no code dependency on `frf-authz-keto`.

→ **Verdict for the spec:** 2 Keto tuple seeds via plain HTTP `PUT` before subscribing,
with deterministic envelope `id`. No wildcard, no OPL, no new dependency.

---

## 4. Full recipe for the realtime-receipt proof

The smoke gains 3 new steps (authoring in `smoke/`, nothing into the fabric repo):

1. **`smoke/fabric-config/dev-jwks.json`** — a static JWKS file with a pre-generated RSA
   public key (2048-bit). The matching private key is an artifact the smoke uses to sign.
2. **`smoke/compose.fabric-dev-idp.yml`** (or a compose service override) — adds an `nginx`
   or `python3 http.server` container serving `dev-jwks.json` at a known URL; updates
   `GATEWAY_JWKS_URL` to point at it. Updates `JWT_AUDIENCE` to `"frf-gateway"` (already
   correct in both fabric composes).
3. **In `smoke.real.spec.ts`** — the realtime test:
   - Mint an RS256 JWT (private key in the smoke) with `sub`, `aud: frf-gateway`, `exp`,
     `tenant_id`, `session_id`.
   - Seed the 2 Keto tuples (subscribe + view) via plain HTTP to the Keto write port
     (`:4467` — mapped to host port `:14467`? need to verify/add port mapping).
   - Subscribe to the channel, then publish the envelope with the known `id`, assert the
     SSE bridge emits it.

---

## 5. Gap summary

| # | Gap | Size | Blocker? |
|---|---|---|---|
| G1 | Dev IdP: pre-generated RSA key pair + static JWKS + JWKS server in compose | Small-med | No |
| G2 | Keto port mapping in compose (need :4467 exposed to host for seed step) | Small | No — just a port mapping |
| G3 | JWT minting in the smoke spec (RS256, `jose`/`jsonwebtoken` in Node) | Small | No — `jsonwebtoken` in smoke's devDeps already supports RS256 |
| G4 | Keto seed step in the smoke spec (plain HTTP PUT to :4467) | Small | No |
| G5 | Updated compose wiring (`GATEWAY_JWKS_URL` → the JWKS server) | Small | No |
| G6 | Updated `smoke.real.spec.ts` realtime test (replace auth-boundary proof with receipt proof) | Small-med | No |

---

## 6. One remaining check

Verify `jsonwebtoken` (already in `smoke/package.json` devDeps) supports RS256 JWT
generation from a raw PEM/JWK — so no new npm dep is needed. Confirm `FrfClaims.session_id`
is optional or whether `to_verified_claims` panics on a missing `session_id`.

---

## 7. Handoff to plan

**Both grounding questions answered from the live code.** The dev IdP = static RSA JWKS
served in the compose (Option a, lean). No Keto wildcard: 2 exact tuple seeds before
publish/subscribe, with a deterministic envelope `id` the smoke chooses. `jsonwebtoken`
(already present) supports RS256. The phase is 4-6 small changes; no agent Rust change
(pure smoke composition + the realtime spec update). Key remaining verification:
`FrfClaims.session_id` optionality + the Keto write port mapping.
