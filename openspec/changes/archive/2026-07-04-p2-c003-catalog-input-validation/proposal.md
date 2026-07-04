## Why

Catalog entries carry no input schema, so `AppError::InvalidInput` is never
raised and MCP `tools/list` advertises a bare `{"type":"object"}`. Task inputs are
unvalidated before dispatch. This change adds per-kind input schemas and validates
against them.

## What Changes

- Extend `CatalogEntry` with an input JSON Schema (per kind).
- Validate `task.input` against the entry's schema in `TaskRunner::run` before dispatch → `AppError::InvalidInput` on mismatch.
- Surface the real per-kind `inputSchema` in MCP `tools/list` (replacing the placeholder).
- Seed schemas for the read kinds (`project.inspect` needs a project id; `forge.table.describe` needs a table name; `project.list`/`forge.table.list` take no required input).

## Capabilities

### New Capabilities
- `catalog-input-validation`: Per-kind input schemas on the catalog, validated before dispatch, and surfaced through MCP `tools/list`.

### Modified Capabilities

## Impact

- `fpa-app` (`CatalogEntry` gains a schema; `TaskRunner` validates), `fpa-gateway` (MCP `tools/list` uses real schemas).
- Small validation dep may be needed (e.g. `jsonschema`) — verify at execute; prefer a minimal shape-check if a full validator is overkill.
- Independent of the forge changes (can run in parallel).

## Open Questions
- Validation library vs hand-rolled shape check: adopt a minimal validator only if the schemas justify it (Base Rule 22/27). Decide at execute.
