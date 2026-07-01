## ADDED Requirements

### Requirement: Table metadata from forge OpenAPI

`fpa-forge::list_tables` SHALL retrieve fabric table/entity metadata from forge's
public `GET /openapi.json` and return it, without requiring a bearer.

#### Scenario: List returns forge metadata

- **WHEN** `list_tables` is called and forge serves a compiled OpenAPI document
- **THEN** the adapter returns the table/entity list derived from that document

#### Scenario: Forge unreachable

- **WHEN** forge cannot be reached
- **THEN** the adapter returns `PortError::Transport`, not a panic

### Requirement: Data reads under RLS with the operator bearer

When a forge read requires data (not just metadata), `fpa-forge` SHALL call
`POST /graphql` with a standard GraphQL body and forward the operator's bearer as
the `Authorization` header, so forge applies RLS.

#### Scenario: Authorized data read

- **WHEN** a data read runs with a valid operator bearer
- **THEN** the adapter sends `Authorization: Bearer <token>` to `/graphql` and returns the result

#### Scenario: Missing/invalid credential

- **WHEN** forge responds 401 (missing or invalid bearer)
- **THEN** the adapter returns `PortError::Unauthorized`

### Requirement: Reads are side-effect free this phase

`fpa-forge` SHALL NOT perform forge mutations in this phase; write-oriented task
kinds return a handled `PortError` indicating the write API is pending.

#### Scenario: Write kind is refused cleanly

- **WHEN** a write-oriented forge task (e.g. `project.create`) is dispatched
- **THEN** the adapter returns `PortError::Downstream` noting writes are not yet implemented, and performs no mutation
