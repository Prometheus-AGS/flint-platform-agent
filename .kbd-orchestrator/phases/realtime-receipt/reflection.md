# Reflection — realtime-receipt

**Phase:** realtime-receipt (p12)
**Stage:** reflect
**Date:** 2026-07-08
**Backend:** OpenSpec (`openspec/changes/p12-c00*`)

---

## Goal achievement

| # | Goal | Verdict | Evidence |
|---|------|---------|----------|
| 1 | Prove the agent **receives** a fabric `EventEnvelope` end-to-end (write → fabric publish → CDC/subscribe → agent `/fabric/subscribe` SSE) | **PARTIAL** | The agent side is proven and the full auth path exercises green up to fabric's broker; the final receipt is blocked by an upstream fabric bug (below). 4/5 real hops pass; the receipt test is `test.fixme`. |
| 2 | Smoke-owned dev IdP (RS256 JWKS the fabric gateway trusts via `GATEWAY_JWKS_URL`), nothing written into `../flint-realtime-fabric` | **MET** | c001 ships an RSA-2048 keypair + `jwks.json` (`kid=dev-smoke-key-1`, RS256), served by an `nginx:alpine` service (c002) at `http://dev-idp-jwks:8080/jwks.json`; fabric verifies minted bearers on both hops. All artifacts live under `smoke/`. |
| 3 | Ground the Keto tuple model before speccing (wildcard/subject-set vs post-publish seed) | **MET** | Grounded against `frf-authz-keto`; the spec seeds three concrete tuples keyed off the RS256 `sub` (subscribe/publish/view) via the verified `PATCH :4467/admin/relation-tuples` insert format, before subscribing. |

**Overall: 2 MET, 1 PARTIAL.** The one PARTIAL is a genuine, root-caused,
non-agent blocker — not incomplete agent work.

## Delivered changes (3/3)

| Change | What landed | Status |
|--------|-------------|--------|
| `p12-c001-dev-idp-keys` | Throwaway RS256 dev IdP: RSA-2048 keypair, JWKS, README, `mint-fabric-bearer.mjs`. Private key intentionally committed (smoke-only). | DONE |
| `p12-c002-compose-dev-idp` | `dev-idp-jwks` nginx service + Keto host ports (14466/14467) wired into `compose.fabric.yml` + `compose.real.yml`; fabric `GATEWAY_JWKS_URL` → dev IdP. | DONE |
| `p12-c003-realtime-receipt-spec` | RS256 bearer mint + 3 Keto seeds + subscribe→publish→receipt assertion in `smoke.real.spec.ts`. | DONE (verification PARTIAL — see below) |

**Agent seam:** `FPA_FABRIC_BEARER` — the `/fabric/subscribe` bridge forwards a
configured RS256 bearer to fabric instead of the operator's bearer when the two
IdPs differ; falls back to the operator's bearer when unset. Redacted in `Debug`;
never logged. Backward-compatible (`None` ⇒ prior behavior). `cargo check -p
fpa-gateway` clean.

## Verification result

`smoke/run-real.sh --no-build` against the full real stack (agent + real
flint-gate + real fabric + Iggy + Keto + agent Postgres; RS256 on **both** the
subscribe and publish hops; **no `DEV_NO_AUTH`**):

```
✓ agent healthz is up
✓ unauthenticated protected surfaces are rejected
✓ fabric.health flows through to the REAL fabric gateway
✓ project CRUD round-trips through the live agent store (real Postgres)
- realtime: agent receives a fabric EventEnvelope end-to-end   (skipped — test.fixme)
4 passed · 1 skipped · OK: real smoke passed
```

## Blocker (upstream, root-caused)

**KI-1 / `Prometheus-AGS/flint-realtime-fabric#2`.** In `frf-broker-iggy`,
subscribe and publish target **different Iggy streams/topics**:

- `publish` / `ensure_channel` → stream `tenant-{tenant_id}`, topic `topic_name(path)`
- `subscribe` (`broker.rs:129`) → stream `channel-{channel_id}`, topic `"events"` (hardcoded)

Nothing creates `channel-<uuid>`, so against a real Iggy `consumer.init()` fails
`Stream not found` → the WS upgrade fails → the agent bridge returns HTTP 502.
The same divergence exists in fabric's own `#[ignore]`d
`tests/publish_subscribe.rs`, so this path has **never run green in CI** upstream.
The failure is entirely inside fabric's broker adapter; the agent opens the WS
and forwards a valid RS256 bearer that fabric verifies before erroring.

Documented in `smoke/KNOWN-ISSUES.md` (KI-1). Un-block: flip `test.fixme` → `test`
and re-run for a true 5/5 once fabric ships the naming fix.

## Artifact Quality Summary

| Metric | Value |
| --- | --- |
| Changes with QA (artifact-refiner) | 0/3 |
| First-pass pass rate | n/a (QA gate not run) |
| Changes requiring refinement | 0 |

QA gate not invoked: c001/c002 are infra/config (composes, static keys); c003 is
a single TypeScript spec file. Validation was the real integration smoke itself
(the phase's stated milestone), which is stronger signal here than per-artifact
refinement. No recurring constraint violations.

## Technical debt introduced

- **`test.fixme` on the receipt test.** A real, tracked, single-line debt with an
  explicit un-block trigger and a filed upstream issue. Not silent — it reports
  SKIPPED and is documented in KNOWN-ISSUES.md and the spec header. **Owner:**
  fabric (#2). **Cost to us on their fix:** flip one marker + re-run.
- **OpenSpec changes not archived.** c001–c003 remain in `openspec/changes/`
  (not `/opsx:verify`+`/opsx:archive`) because c003's verification is honestly
  PARTIAL. Archive after fabric#2 lands and the smoke reaches 5/5, or by explicit
  operator decision to archive the 4/5 proof as-is.
- **Committed throwaway RSA private key** (`smoke/dev-idp/private-key.pem`) —
  intentional, documented, smoke-only. Not debt so much as a deliberate,
  clearly-labeled tradeoff (mirrors the committed HS256 secret).

## Lessons captured

- **"Real everything" surfaces latent upstream bugs the mocked path hid.** The
  fabric broker's stream-name divergence was invisible until a genuinely-real
  Iggy rejected the unknown stream. Value of the no-`DEV_NO_AUTH`, real-broker
  smoke: it found a bug fabric's own CI can't (their test is `#[ignore]`d).
- **When a sibling repo you're constrained not to edit is the root cause, the
  correct move is: root-cause it precisely, file it, mark the blocked test
  auditable (`test.fixme`, not deletion/skip-silently), and commit the proof
  that *does* work.** Never fake green.
- **The `FPA_FABRIC_BEARER` seam is the reusable win** — a clean
  forward-a-different-IdP-token pattern for any downstream plane whose IdP
  differs from the agent's, with a safe fallback and no secret leakage.

## Recommended next phase

**`realtime-receipt-unblock`** (small, fabric-gated): watch
`Prometheus-AGS/flint-realtime-fabric#2`; when the broker naming fix ships, flip
`test.fixme` → `test`, re-run `run-real.sh --no-build` for a true 5/5, then
`/opsx:verify` + `/opsx:archive` c001–c003. Until then this phase is
**complete-with-documented-upstream-block** — do not spin further agent work on
receipt; the agent side is proven.

Alternative if fabric#2 lingers: proceed to the next unrelated administrative
surface (A2A task catalog expansion / gate route admin) and let the unblock ride
as a background watch.
