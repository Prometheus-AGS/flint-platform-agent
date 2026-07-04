## 1. Schema on the catalog

- [x] 1.1 Extend `CatalogEntry` with an `input_schema` (JSON Schema as a static/`Value`).
- [x] 1.2 Seed schemas: `project.inspect` (requires `project_id`), `forge.table.describe` (requires `name`); list kinds take no required input.

## 2. Validation in the runner

- [x] 2.1 Validate `task.input` against the entry schema in `TaskRunner::run` before dispatch.
- [x] 2.2 On mismatch return `AppError::InvalidInput`; call no port.
- [x] 2.3 Decide validator: minimal hand-rolled required-field check vs a `jsonschema` crate (verify version/MSRV at execute; prefer minimal).

## 3. Surface schemas via MCP

- [x] 3.1 `tools/list` uses each entry's `input_schema` instead of the placeholder `{"type":"object"}`.

## 4. Verification

- [x] 4.1 `cargo check/clippy/fmt` green.
- [x] 4.2 Test: missing required field → `AppError::InvalidInput`, no port call.
- [x] 4.3 Test: valid input passes validation.
- [x] 4.4 Test: `tools/list` advertises the real per-kind schema.
