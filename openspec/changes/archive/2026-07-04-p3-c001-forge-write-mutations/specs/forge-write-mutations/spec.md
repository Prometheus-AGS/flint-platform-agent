## ADDED Requirements

### Requirement: Forge writes via pg_graphql mutations under the bearer

`fpa-forge` SHALL perform forge writes by POSTing a pg_graphql mutation to
`/graphql` with the operator's bearer as `Authorization: Bearer`. The agent MUST
NOT replicate forge's authorization — forge (Keto + Cedar) is the authority.

#### Scenario: Authorized write

- **WHEN** a write task runs with a valid operator bearer and forge accepts it
- **THEN** the adapter sends the mutation with the bearer and returns forge's result

#### Scenario: Missing bearer

- **WHEN** a write task runs without an operator bearer
- **THEN** the adapter returns `PortError::Unauthorized` and sends no mutation

### Requirement: Policy denial is distinct from missing identity

A forge **403** (Keto/Cedar policy denial) SHALL map to `PortError::Unauthorized`,
distinguishable from a 401 (missing/invalid bearer) in the message.

#### Scenario: Forge denies by policy

- **WHEN** forge responds 403 to a mutation
- **THEN** the adapter returns `PortError::Unauthorized` noting a policy denial

### Requirement: Unimplemented writes fail cleanly

Any write-oriented task kind without a real forge mutation SHALL return
`PortError::Downstream` indicating the write API is pending, and MUST NOT fall
through to a read operation.

#### Scenario: Unimplemented write kind

- **WHEN** a write kind with no mutation mapping is dispatched
- **THEN** the runner returns a downstream "write API pending" error and performs no read
