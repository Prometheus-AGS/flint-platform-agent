## Why

`crates/fpa-app/src/task_runner.rs` is **828 lines** — already over the project's
500-line hard file limit (a CI-enforced gate). This is carried debt, not introduced
here, but the next two changes in this phase (`p13-c002`, `p13-c003`) add dispatch arms
and integration tests to this file, which would push it further over the limit. Split it
into a directory module **first** so subsequent changes land in a compliant file.

## What Changes

Convert `crates/fpa-app/src/task_runner.rs` (single file) into a directory module:

- `crates/fpa-app/src/task_runner/mod.rs` — the production body (current lines 1–341),
  moved verbatim. Declares `#[cfg(test)] mod tests;`.
- `crates/fpa-app/src/task_runner/tests.rs` — the `#[cfg(test)] mod tests { … }` block
  (current lines 342–828), moved out. The inner `use` becomes `use super::*;` (the tests
  already reference `TaskRunner`, the `Fake*` doubles, `catalog`, `TargetPort` — all
  reachable via `super`).

This is a **mechanical, behavior-preserving** extraction. No production logic changes,
no public API changes: `crate::task_runner::TaskRunner` still resolves. After the split
both files are under 500 lines (~341 prod, ~490 test).

## Capabilities

### New Capabilities
- (none — mechanical refactor)

### Modified Capabilities
- `task-catalog`: the task runner is reorganized into a directory module so it stays
  under the 500-line file limit while the catalog grows. Behavior is unchanged.

## Impact

- `crates/fpa-app/src/task_runner.rs` → `crates/fpa-app/src/task_runner/{mod.rs, tests.rs}`.
- No other crate changes; no gateway change; no dependency change.

## Open Questions

- None. Split-by-plane (dispatch-per-target files) was considered and deferred as YAGNI;
  tests-only extraction is the smallest diff that gets both files under the limit.
