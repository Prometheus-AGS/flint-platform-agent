## 1. Retain the raw bearer

- [x] 1.1 In `fpa-gateway/identity.rs`, keep the raw bearer string on `OperatorContext` (do not discard after decode).
- [x] 1.2 Ensure no `Debug`/log path prints the raw bearer (audit the derive + tracing).

## 2. Thread through the app layer

- [x] 2.1 Add `bearer: Option<String>` to `fpa_app::AuthContext`.
- [x] 2.2 Make the bearer available inside `TaskRunner::run` to the dispatched port (pass to the forge dispatch path).
- [x] 2.3 Confirm `#[tracing::instrument(skip_all, …)]` still excludes the bearer; add an explicit test asserting logs contain no token.

## 3. Wire the surfaces

- [x] 3.1 A2A `submit`: populate `AuthContext.bearer` from `OperatorContext`.
- [x] 3.2 MCP `tools/call`: build `AuthContext` (roles + bearer) from the caller's gate identity on `POST /mcp` (Q4 — if MCP carries no JWT, keep a scoped default and note the gap).

## 4. Verification

- [x] 4.1 `cargo check/clippy/fmt` green.
- [x] 4.2 Test: `AuthContext` built from a gate JWT carries the exact bearer.
- [x] 4.3 Test: no-bearer request yields `AuthContext` with `bearer: None`.
- [x] 4.4 Test: audit/log output for a run contains no bearer substring.
