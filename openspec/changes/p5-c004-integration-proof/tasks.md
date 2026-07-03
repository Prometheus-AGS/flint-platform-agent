## 1. Harness scaffolding

- [ ] 1.1 Add a `tests/` integration binary (in `fpa-gateway`, or a dev-only `fpa-integration` crate). Wire the real router via `AppState` with a test config.
- [ ] 1.2 Add wiremock mock servers for forge, gate, fabric; point the adapters' base URLs at them via config.
- [ ] 1.3 Ephemeral RSA keypair in-test (verify the crate version first — `rcgen`/`rsa`/jsonwebtoken helper); serve its public JWK via a wiremock JWKS endpoint; set `FPA_JWKS_URL` to it.

## 2. Drive the flow (assert each hop)

- [ ] 2.1 **Authenticate:** sign a bearer with the in-test key; assert it's accepted (`signature_verified = true`) and an unauthenticated call is rejected — incl. `GET /agui/stream`.
- [ ] 2.2 **project.create (A2A submit):** assert a real `Project` is stored and returned; assert forge mock received **no** write.
- [ ] 2.3 **list_routes (Gate read kind):** assert the mocked gate routes come back; assert `application.deploy` is refused (gate mock receives no route-write).
- [ ] 2.4 **fabric.health:** assert ok from the fabric mock.
- [ ] 2.5 **MCP:** assert `tools/list` then a `tools/call` dispatch round-trips through the MCP client mock.

## 3. Single-flight JWKS

- [ ] 3.1 Concurrent cold-cache verify → assert the wiremock JWKS endpoint received exactly one request.

## 4. Verification (the phase's justified cargo test milestone)

- [ ] 4.1 `cargo check/clippy/fmt` green across the workspace (batched — all of p5-c001..c004 implemented first).
- [ ] 4.2 **One full `cargo test`** run (integration milestone #1 of ≤3 for this goal). Fix to green.
- [ ] 4.3 Document the live-smoke follow-on (real forge/gate/fabric compose) as out-of-scope, in the change or reflection.
