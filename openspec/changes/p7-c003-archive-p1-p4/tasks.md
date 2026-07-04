## 1. Reconcile the 2 partials

- [x] 1.1 Read `p2-c002-forge-read-integration` tasks.md (12/13): if the trailing task is genuinely done, mark it `[x]`; else note it and skip archival for this change.
- [x] 1.2 Read `p1-c004-mcp-transport` tasks.md (11/12): same reconcile-or-skip.

## 2. Archive the complete changes

- [x] 2.1 `openspec archive` the 12 `✓ Complete` p1–p4 changes (`p1-c001`, `p1-c002`, `p1-c003`, `p2-c001`, `p2-c003`, `p2-c004`, `p3-c001`, `p3-c002`, `p3-c003`, `p4-c001`, `p4-c002`, `p4-c003`) with `--yes`.
- [x] 2.2 Archive any partial that got reconciled in step 1.

## 3. Verification

- [x] 3.1 `openspec list` shows the archived changes gone from active; `openspec/specs/` has their capabilities.
- [x] 3.2 `git status` shows only `openspec/` moves (no Rust source touched by the archival). Any skipped partial is noted in the reflection.
