## 1. Archive p6 changes

- [ ] 1.1 `openspec archive p6-c001-durable-project-store --yes`.
- [ ] 1.2 `openspec archive p6-c002-archive-p5-changes --yes`.

## 2. Verification

- [ ] 2.1 `openspec list` shows the two p6 changes as archived; `openspec/specs/` contains the durable-project-store capability.
- [ ] 2.2 `git status` shows only `openspec/` moves (no Rust source touched).
