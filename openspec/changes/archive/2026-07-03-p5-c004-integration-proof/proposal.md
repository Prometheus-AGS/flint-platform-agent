## Why

All four fabric planes are implemented and auth is hardened, but the system has
**never run end-to-end**. Per the implementation-first philosophy, once the whole
system is present we prove the *whole shape* — not isolated units. There are zero
integration tests today. This change is the phase's spine: one end-to-end proof
across AG-UI / A2A / MCP, and the **first justified `cargo test`** of the phase.

## What Changes

- Add an integration test (a new `tests/` binary in `fpa-gateway`, or a dedicated
  `fpa-integration` dev-only harness) that boots the **real** Axum router with
  planes mocked at the HTTP boundary and a **real** in-memory `ProjectStore`.
- Mock forge/gate/fabric with `wiremock` (already a dev-dep). Mint an **ephemeral
  RSA keypair in-test** and serve its public JWK via wiremock as the IdP for the
  verify path.
- Drive one full operator flow and assert each hop:
  1. **authenticate** — a request with a signature-verifiable bearer is accepted;
     an unauthenticated one is rejected (including `/agui/stream`).
  2. **project.create (A2A)** — stores a real `Project` in the `ProjectStore` and
     returns it (no forge write).
  3. **list_routes (gate, via a Gate read kind)** — returns the mocked gate routes;
     `application.deploy` is refused.
  4. **fabric.health** — returns ok from the mocked fabric.
- Exercise the surfaces: A2A `submit`/`status`, the MCP `tools/list`+`tools/call`
  dispatch, and the AG-UI `/agui/stream` auth gate.

Live smoke against real siblings (forge Postgres-18 image + gate + fabric compose)
is a documented **follow-on**, not this change.

## Capabilities

### New Capabilities
- `integration-proof`: An end-to-end integration test that proves the authenticate → project.create → list_routes → fabric.health flow across AG-UI/A2A/MCP with planes mocked at the HTTP boundary and a real ProjectStore + in-test JWKS.

### Modified Capabilities

## Impact

- New `tests/` harness (dev-only); adds dev-deps only if needed (`rsa`/`rcgen` or
  `jsonwebtoken`'s key gen for the ephemeral keypair — verify current version first
  per Base Rule 22). `wiremock`, `tokio`, `axum` test client already present.
- No production-code change (relies on p5-c001/002/003 landing first).

## Open Questions
- **RESOLVED (analyze):** mock-at-boundary first; real in-memory ProjectStore;
  in-test ephemeral RSA JWKS. Live smoke deferred. Which prod IdP backs
  `FPA_JWKS_URL` stays deployment config (out of scope).
- Ephemeral-key crate choice (`rcgen` vs `rsa` vs jsonwebtoken helper) — pick the
  smallest already-compatible option at execute time; verify the version.
