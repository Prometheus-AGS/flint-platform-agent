## Why

The four phase-5 OpenSpec changes (`p5-c001`..`p5-c004`) are implemented, CI-green,
and pushed, but not yet archived — so `openspec/specs/` does not yet reflect the
capabilities they shipped (p5 debt #3). Archiving folds each change's spec delta into
the baseline and moves the change under `openspec/changes/archive/`.

## What Changes

- `/opsx:archive` (or the equivalent `openspec archive`) for each of `p5-c001-project-store`,
  `p5-c002-forge-rest-path-fix`, `p5-c003-security-debt-closure`, `p5-c004-integration-proof`,
  applying each `ADDED Requirements` delta into `openspec/specs/`.
- No source-code change; this is spec-baseline housekeeping.

## Capabilities

### New Capabilities

### Modified Capabilities
- `spec-baseline`: The `openspec/specs/` baseline reflects the shipped phase-5 capabilities (project-store, forge-rest-path, security-debt-closure, integration-proof) after archival.

## Impact

- `openspec/specs/` (gains the p5 capability specs) and `openspec/changes/archive/`
  (gains the four archived changes). No Rust code touched.

## Open Questions
- None. Purely mechanical archival of already-validated, already-shipped changes.
