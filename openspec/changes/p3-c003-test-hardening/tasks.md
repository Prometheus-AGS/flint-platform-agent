## 1. Bearer threading unit tests

- [ ] 1.1 `fpa-gateway` (or `fpa-app`): test that an `AuthContext` built from a gate-JWT operator carries the exact bearer.
- [ ] 1.2 Test that a no-identity path yields `AuthContext.bearer == None`.

## 2. MCP schema advertisement test

- [ ] 2.1 Test `tool_definitions()` (or the MCP route) advertises the real per-kind `inputSchema` — assert `forge.table.describe` exposes required `name`.

## 3. Redaction test

- [ ] 3.1 Test that `Debug` of the bearer-carrying context omits the token and shows `<redacted>` (may already exist for `OperatorContext`; add for `AuthContext`).

## 4. Verification

- [ ] 4.1 `cargo check/clippy/fmt` green; all new tests pass.
