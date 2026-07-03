# integration-proof Specification

## Purpose
TBD - created by archiving change p5-c004-integration-proof. Update Purpose after archive.
## Requirements
### Requirement: End-to-end operator flow is proven across surfaces

An integration test SHALL boot the real Axum router with forge/gate/fabric mocked
at the HTTP boundary and a real in-memory `ProjectStore`, and drive the flow
`authenticate → project.create → list_routes → fabric.health` end-to-end, asserting
each hop's outcome.

#### Scenario: Happy-path flow

- **WHEN** an authenticated operator submits `project.create`, then a Gate read task, then `fabric.health`
- **THEN** the project is stored and returned, the gate routes are returned, and fabric health is ok — each via the agent's real handler stack

### Requirement: Authentication is enforced end-to-end

The proof SHALL assert that unauthenticated requests are rejected — including
`GET /agui/stream` — and that a signature-verifiable bearer (against an in-test
JWKS) is accepted.

#### Scenario: Unauthenticated request rejected

- **WHEN** a protected surface is called with no valid identity
- **THEN** the request is rejected before any downstream effect

#### Scenario: Verified bearer accepted

- **WHEN** a bearer signed by the in-test key is presented and the JWKS serves the matching public key
- **THEN** the identity is accepted with `signature_verified = true`

### Requirement: Write refusals hold under the full stack

The proof SHALL assert that `application.deploy` is refused (no gate route-write)
and that `project.create` performs no forge write.

#### Scenario: Deploy refused end-to-end

- **WHEN** `application.deploy` is dispatched through A2A
- **THEN** the runner returns a downstream "not implemented" error and the gate mock receives no route-write request

### Requirement: JWKS is fetched at most once under concurrency

The proof (or a focused test in this change) SHALL assert single-flight JWKS
behavior: concurrent verifications on a cold cache produce exactly one IdP fetch.

#### Scenario: Concurrent verify, single fetch

- **WHEN** two verifications race against a cold JWKS cache
- **THEN** the wiremock JWKS endpoint receives exactly one request

