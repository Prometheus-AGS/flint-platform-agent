## ADDED Requirements

### Requirement: Forge REST insert addresses schema and table at the gateway root

`fpa-forge` SHALL POST a REST insert to `{base}/{schema}/{table}` — the schema and
table as two path segments, with **no `/rest` prefix** (forge merges the REST router
at the gateway root). The adapter MUST NOT construct a single-segment
`/rest/<table>` path.

#### Scenario: Authorized REST insert

- **WHEN** a forge write runs for schema `s`, table `t`, with a valid bearer
- **THEN** the adapter POSTs to `{base}/{s}/{t}` with the bearer and returns forge's result

#### Scenario: Path arity

- **WHEN** the adapter builds an insert URL
- **THEN** the URL contains both the schema and the table segment, in that order, and no `/rest` segment

### Requirement: Status mapping is preserved

The corrected path SHALL keep the existing status mapping: 2xx → success; 401/403 →
`PortError::Unauthorized`; other non-2xx → `PortError::Downstream`; unreachable →
`PortError::Transport`; missing bearer → `PortError::Unauthorized` (no request sent).

#### Scenario: Policy denial

- **WHEN** forge responds 403 to the insert
- **THEN** the adapter returns `PortError::Unauthorized`

#### Scenario: Missing bearer

- **WHEN** no operator bearer is present
- **THEN** the adapter returns `PortError::Unauthorized` and sends no request
