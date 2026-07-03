# security-debt-closure Specification

## Purpose
TBD - created by archiving change p5-c003-security-debt-closure. Update Purpose after archive.
## Requirements
### Requirement: The AG-UI stream requires an authenticated operator

`GET /agui/stream` SHALL require an `OperatorContext` (via the identity extractor).
An unauthenticated request MUST be rejected before any event is streamed.

#### Scenario: Unauthenticated stream request

- **WHEN** `GET /agui/stream` is requested with no trusted headers and no valid bearer
- **THEN** the request is rejected (no SSE stream is opened)

#### Scenario: Authenticated stream request

- **WHEN** `GET /agui/stream` is requested with a valid operator identity
- **THEN** the SSE stream opens and emits the run bracket

### Requirement: Gate write kinds refuse instead of listing routes

A `TargetPort::Gate` write kind SHALL return `PortError::Downstream` indicating gate
route-writes are not implemented, and MUST NOT call `list_routes()`. Only
read-oriented Gate kinds (e.g. a route-list read) call `list_routes()`;
`application.deploy` is a write and is refused.

#### Scenario: application.deploy is refused

- **WHEN** an authorized operator dispatches `application.deploy`
- **THEN** the runner returns a downstream "gate route-write not implemented" error and does not list routes

### Requirement: JWKS refresh is single-flight

`JwksVerifier` SHALL serialize JWKS refresh so that concurrent callers on a
cold or stale cache trigger **at most one** IdP fetch; queued callers reuse the
freshly-cached key set.

#### Scenario: Concurrent cold-cache verification

- **WHEN** two verifications race with an empty cache
- **THEN** only one HTTP fetch to the IdP occurs and both verifications use the same fetched key set

### Requirement: The task audit records signature provenance

The task audit record SHALL include whether the operator identity was
signature-verified (direct token) or gate-trusted (injected headers). The token,
claims, and secrets MUST NOT be logged.

#### Scenario: Audit distinguishes verified from trusted

- **WHEN** a task runs under a signature-verified identity
- **THEN** the audit log records `signature_verified = true` (and `false` for a gate-trusted identity), with no token or claim values

