## Why

The phase-6 OpenSpec changes (`p6-c001-durable-project-store`,
`p6-c002-archive-p5-changes`) are shipped and `✓ Complete` but not yet archived —
`openspec/specs/` doesn't reflect their capabilities and they still sit in the active
list. The p7 reflection flagged this as trivial housekeeping to fold in.

## What Changes

- `openspec archive` `p6-c001-durable-project-store` and `p6-c002-archive-p5-changes`
  into `specs/`.
- No source-code change.

## Capabilities

### New Capabilities

### Modified Capabilities
- `spec-baseline`: `openspec/specs/` reflects the shipped phase-6 capabilities (durable-project-store) after archival; the active changes list is cleared of the completed p6 changes.

## Impact

- `openspec/specs/` (+ p6 capability) and `openspec/changes/archive/`. No Rust touched.

## Open Questions
- None. Mechanical archival of already-shipped, validated changes.
