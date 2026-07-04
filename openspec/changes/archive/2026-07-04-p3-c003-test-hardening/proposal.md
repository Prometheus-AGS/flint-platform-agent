## Why

Phase 2 marked several test tasks complete that were verified via live smoke
rather than dedicated unit tests (reflection debt #2). This change adds the
missing unit tests so the coverage claims are honest and the behaviors are
regression-guarded.

## What Changes

- Unit test: `AuthContext` built from a gate JWT carries the exact bearer; a
  no-identity request yields `bearer: None` (phase-2 c001 tasks 4.2/4.3).
- Unit test: MCP `tools/list` advertises each catalog kind's **real** input
  schema, not a placeholder (phase-2 c003 task 4.4).
- Unit test: `OperatorContext`/`AuthContext` `Debug` redacts the bearer (guards
  the no-token-in-logs guarantee).
- No production behavior change — tests only.

## Capabilities

### New Capabilities

### Modified Capabilities

## Impact

- Test-only additions across `fpa-gateway` (identity, MCP route) and `fpa-app`.
- No new dependencies. Documentation-adjacent; small.

## Note
This is a test-only change (no `specs/` delta). It exists to close the honest
coverage gap flagged in the phase-2 reflection, not to add capability.
