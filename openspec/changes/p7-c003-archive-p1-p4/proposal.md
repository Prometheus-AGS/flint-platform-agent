## Why

The p1–p4 OpenSpec changes are shipped but never archived — they still clutter the
active `openspec/changes/` list and their capabilities aren't in `openspec/specs/`.
p5 was archived in p6-c002; this clears the earlier backlog. Two changes show partial
task counts (`p2-c002-forge-read-integration` 12/13, `p1-c004-mcp-transport` 11/12)
and need a trailing-task look before they can archive cleanly.

## What Changes

- Inspect the two partials (`p2-c002`, `p1-c004`): if the trailing task is genuinely
  done (likely a verification checkbox), mark it and archive; otherwise **skip + note**
  the partial for a later reconcile (do NOT force-archive an incomplete change).
- `openspec archive` every **`✓ Complete`** p1–p4 change into `specs/` (the 12
  complete ones, plus any partial that gets reconciled).
- No source-code change; spec-baseline housekeeping.

## Capabilities

### New Capabilities

### Modified Capabilities
- `spec-baseline`: `openspec/specs/` reflects the shipped p1–p4 capabilities after archival; the active changes list is cleared of completed earlier changes.

## Impact

- `openspec/specs/` (+ p1–p4 capabilities) and `openspec/changes/archive/`. No Rust
  touched.

## Open Questions
- **Per-partial (assess lean):** reconcile-if-trivial, else skip + note. Decided per
  change at execute after reading its trailing task.
