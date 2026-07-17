## MODIFIED Requirements

### Requirement: forge.table.describe forwards the requested table name

`forge.table.describe` SHALL pass the caller-supplied `table` name (from the validated
task input) to `ForgeMetadata::describe_table`. It MUST NOT substitute a placeholder or
discard the name. A missing or empty `table` after schema validation MUST return
`AppError::InvalidInput`, not a port call with a placeholder.

#### Scenario: Describe forwards the exact table name

- **WHEN** `forge.table.describe` runs with input `{"table":"widgets"}`
- **THEN** `describe_table` is called with `"widgets"` (not `"<unspecified>"`)
- **AND** the returned description is for the requested table

#### Scenario: Empty table name is rejected

- **WHEN** `forge.table.describe` somehow reaches dispatch with an empty/absent `table`
- **THEN** it returns `AppError::InvalidInput`
- **AND** it does not call `describe_table` with a placeholder

#### Scenario: The forwarding is guarded by a test

- **WHEN** the task-runner test suite runs
- **THEN** a test asserts `describe_table` received the exact requested name, so a
  regression to the placeholder would fail the build
