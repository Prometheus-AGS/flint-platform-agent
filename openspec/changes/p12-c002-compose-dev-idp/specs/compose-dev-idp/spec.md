## ADDED Requirements

### Requirement: The fabric compose stacks include a JWKS server and expose Keto's write port

Both `smoke/compose.fabric.yml` and `smoke/compose.real.yml` SHALL include a `dev-idp-jwks`
service that serves `smoke/dev-idp/jwks.json` as a static HTTP response. The fabric
`gateway` service SHALL have `GATEWAY_JWKS_URL` pointing at this service. The `fabric-keto`
service SHALL expose port 4467 (write) and 4466 (read) to the host.

#### Scenario: OryIdentityVerifier resolves the dev JWKS

- **WHEN** the fabric gateway starts with `GATEWAY_JWKS_URL=http://dev-idp-jwks:8080/jwks.json`
- **THEN** the verifier fetches the file and verifies RS256 bearers signed with the dev key

#### Scenario: Keto write port is reachable from the host

- **WHEN** the smoke Playwright test calls `PUT http://localhost:14467/relation-tuples`
- **THEN** the Keto write API accepts the tuple (the host port maps to the container's 4467)
