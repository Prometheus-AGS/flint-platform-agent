## ADDED Requirements

### Requirement: A smoke-owned RSA key pair enables RS256 JWT verification by the fabric gateway

`smoke/dev-idp/` SHALL contain a pre-generated 2048-bit RSA key pair: `private-key.pem`
(PEM format, no passphrase) and `jwks.json` (JWKS format, single key with a `kid`). The
JWKS SHALL be servable as a static file from the compose stack so that fabric's
`OryIdentityVerifier` can fetch and verify bearers the smoke mints with the private key.

#### Scenario: The JWKS resolves the public key

- **WHEN** `OryIdentityVerifier` fetches the JWKS URL and a bearer has a matching `kid`
- **THEN** the public key is found and the RS256 signature is verified

#### Scenario: The keys are recognized as smoke-only artifacts

- **WHEN** a reviewer inspects the committed private key
- **THEN** `smoke/dev-idp/README` explains these are throwaway smoke artifacts (not real
  credentials — same precedent as the HS256 test secret in the composes)
