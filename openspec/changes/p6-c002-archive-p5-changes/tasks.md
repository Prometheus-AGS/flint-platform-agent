## 1. Archive p5 changes

- [x] 1.1 `openspec archive p5-c001-project-store --yes` (folds its delta into `specs/`).
- [x] 1.2 `openspec archive p5-c002-forge-rest-path-fix --yes`.
- [x] 1.3 `openspec archive p5-c003-security-debt-closure --yes`.
- [x] 1.4 `openspec archive p5-c004-integration-proof --yes`.

## 2. Verification

- [x] 2.1 `openspec list` shows the four p5 changes as archived; `openspec/specs/` contains their capabilities.
- [x] 2.2 `git status` shows only `openspec/` moves (no Rust source touched).
