# forge-rest-sync Specification

## Purpose
TBD - created by archiving change p4-c003-forge-rest-sync. Update Purpose after archive.
## Requirements
### Requirement: Entity creation via forge REST insert

`fpa-forge::create_entity` SHALL create a row by POSTing the object to forge's REST
table endpoint under the operator bearer, so forge applies RLS + Keto/Cedar.

#### Scenario: Authorized REST insert

- **WHEN** `create_entity` runs with a valid bearer and forge accepts the insert
- **THEN** the adapter POSTs to the table endpoint with the bearer and returns forge's result

#### Scenario: Missing bearer

- **WHEN** `create_entity` runs with no bearer
- **THEN** the adapter returns `PortError::Unauthorized` and sends no request

### Requirement: REST status mapping

Forge REST responses SHALL map onto `PortError`: 2xx (incl. `201`) → success;
`401`/`403` → `Unauthorized`; other non-2xx → `Downstream`; transport failure →
`Transport`.

#### Scenario: Policy denial

- **WHEN** forge responds 403 to a REST insert
- **THEN** the adapter returns `PortError::Unauthorized`

### Requirement: REST path is configuration

The forge REST path prefix SHALL come from configuration (not a hardcoded literal),
so a prefix change is a config fix.

#### Scenario: Configured prefix

- **WHEN** the forge REST prefix is provided via config
- **THEN** the adapter builds table endpoints from it

