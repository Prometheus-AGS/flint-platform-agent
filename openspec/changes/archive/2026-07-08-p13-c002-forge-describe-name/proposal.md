## Why

`forge.table.describe` is a **real, untested bug.** In `dispatch_forge`, the runner calls
`self.forge.describe_table("<unspecified>", bearer)` — the schema-validated `table` name
from the task input is silently discarded and replaced with a placeholder. The
`ForgeMetadata::describe_table(name, bearer)` port takes the real name, so every
`forge.table.describe` task currently describes the wrong (placeholder) table. The bug
survives because the existing test `valid_required_input_passes` only asserts the port was
*called*, never *with what*.

## What Changes

1. **Thread the name.** In `dispatch_forge`, extract the `table` field from the validated
   task input (the `SCHEMA_TABLE_NAME` contract already requires a non-empty `table`
   string) and pass it to `describe_table`, mirroring how `dispatch_store` extracts
   `project_id` for `project.inspect`. If `table` is missing/empty after schema validation
   (defense in depth), return `AppError::InvalidInput` rather than calling the port with a
   placeholder.
2. **Assert the argument.** Extend the `FakeForge` test double to capture the requested
   table name, and add/upgrade a test that asserts `describe_table` is invoked with the
   **exact requested name**, not `"<unspecified>"`. This is the regression guard the bug
   lacked.

No forge write path is touched. No new port, no gateway change, no schema change.

## Capabilities

### Modified Capabilities
- `task-catalog`: `forge.table.describe` correctly forwards the caller-supplied table name
  to the forge metadata port, and the behavior is guarded by an argument-asserting test.

## Impact

- `crates/fpa-app/src/task_runner/mod.rs` (`dispatch_forge` arm).
- `crates/fpa-app/src/task_runner/tests.rs` (`FakeForge` capture + argument-asserting test).
- Depends on `p13-c001` (the runner directory-module split).

## Open Questions

- None.
