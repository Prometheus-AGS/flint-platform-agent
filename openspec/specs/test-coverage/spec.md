# test-coverage Specification

## Purpose
TBD - created by archiving change p3-c003-test-hardening. Update Purpose after archive.
## Requirements
### Requirement: Bearer threading is unit-tested

The credential-threading behavior SHALL have unit tests, not only smoke
verification: `AuthContext` carries the exact bearer from a gate JWT, and a
no-identity request yields `bearer: None`.

#### Scenario: Bearer carried

- **WHEN** an `AuthContext` is built from an operator with a gate bearer
- **THEN** a unit test asserts the `bearer` equals the source token

#### Scenario: No bearer

- **WHEN** an `AuthContext` is built with no gate identity
- **THEN** a unit test asserts `bearer` is `None`

### Requirement: MCP schema advertisement is unit-tested

A unit test SHALL assert that MCP `tools/list` advertises each catalog kind's real
input schema (e.g. `forge.table.describe` requires `name`), not a placeholder.

#### Scenario: Real schema advertised

- **WHEN** the MCP tool list is generated
- **THEN** a unit test asserts a required-field kind exposes its required field in `inputSchema`

### Requirement: Bearer redaction is unit-tested

A unit test SHALL assert the `Debug` output of the identity/auth contexts does not
contain the bearer token.

#### Scenario: Debug redaction

- **WHEN** an auth/operator context containing a bearer is formatted with `Debug`
- **THEN** a unit test asserts the token string does not appear and `<redacted>` does

