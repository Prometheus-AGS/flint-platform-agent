## ADDED Requirements

### Requirement: Gate-injected identity is trusted via configured headers

The agent SHALL trust injected identity without re-verifying when a request carries
the operator-configured trusted identity headers (which gate injects for this
deployment). Trust is deployment-configured; there is no universal gate marker.

#### Scenario: Trusted headers present

- **WHEN** a request carries the configured trusted identity headers
- **THEN** the operator context is built from them without JWKS verification

### Requirement: Directly-received tokens are verified against the IdP JWKS

When no trusted headers are present but a raw bearer is, the agent SHALL verify its
signature against the configured IdP JWKS (RS256/ES256) or the HS256 secret before
trusting it.

#### Scenario: Valid direct token

- **WHEN** a directly-received token verifies against the JWKS
- **THEN** the operator context is built from its claims

#### Scenario: Invalid direct token

- **WHEN** a directly-received token fails verification
- **THEN** the request is unauthenticated (`NoIdentity`)

### Requirement: No unverified decode

A directly-received token SHALL NOT be decoded and trusted without signature
verification. Absence of any usable verification key is a rejection, not an
unverified accept.

#### Scenario: No verification key for a direct token

- **WHEN** a directly-received token arrives and no JWKS/secret is available
- **THEN** the request is rejected (`NoIdentity`), not decoded unverified

### Requirement: JWKS is cached

The agent SHALL cache the fetched JWKS (bounded TTL) rather than fetching per
request.

#### Scenario: Second verification reuses cache

- **WHEN** two direct tokens are verified within the cache TTL
- **THEN** the JWKS is fetched at most once
