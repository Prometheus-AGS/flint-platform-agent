## ADDED Requirements

### Requirement: A Postgres adapter persists the Project aggregate durably

A new `fpa-store-pg` crate SHALL implement `fpa_ports::ProjectStore` over Postgres,
storing the `Project` aggregate **whole** as a JSONB `body` in an agent-owned table.
The adapter MUST round-trip the aggregate losslessly (id, name, `schema_version`, and
every nested collection) and MUST live outside `fpa-app` (the app layer never imports
infrastructure).

#### Scenario: Durable round-trip

- **WHEN** a `Project` is `put` and later `get` by its `ProjectId` from a new adapter instance (fresh connection pool)
- **THEN** the retrieved aggregate equals the stored one, proving persistence survives a process/pool restart

#### Scenario: Upsert on repeated put

- **WHEN** the same `ProjectId` is `put` twice with different contents
- **THEN** the stored row reflects the second value (INSERT … ON CONFLICT DO UPDATE)

#### Scenario: Missing project

- **WHEN** `get` is called with an id that was never stored
- **THEN** the adapter returns `Ok(None)`

### Requirement: The store backend is selected by configuration

The composition root SHALL construct a Postgres-backed `ProjectStore` when
`FPA_PROJECT_DB_URL` is configured, and the in-memory store otherwise. The database
URL MUST NOT appear in any `Debug` output or log.

#### Scenario: Postgres selected when configured

- **WHEN** `FPA_PROJECT_DB_URL` is set at startup
- **THEN** `AppState` wires the Postgres adapter into the `TaskRunner`

#### Scenario: In-memory fallback

- **WHEN** `FPA_PROJECT_DB_URL` is absent
- **THEN** `AppState` wires the in-memory adapter (unchanged behaviour)

#### Scenario: URL is never logged

- **WHEN** the gateway config is formatted for logs/panics
- **THEN** the database URL is redacted

### Requirement: Downstream failures map to the port error surface

The Postgres adapter SHALL map connection/query failures onto `PortError` (transport
vs downstream) and MUST NOT panic on a database error.

#### Scenario: Database unreachable

- **WHEN** the configured database is unreachable during `put`/`get`
- **THEN** the adapter returns a `PortError` (not a panic)
