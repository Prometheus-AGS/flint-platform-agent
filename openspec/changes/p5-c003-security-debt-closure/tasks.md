## 1. G4 — authenticate /agui/stream

- [ ] 1.1 Add `_operator: OperatorContext` to `agui::stream`; keep the stub body. Confirm the extractor rejects unauthenticated requests (same path as A2A status/cancel).

## 2. G3 — gate write-kind guard

- [ ] 2.1 In `task_runner`, for `TargetPort::Gate`, branch on the kind: write kinds (`application.deploy`) → `Err(PortError::Downstream("gate route-write not implemented"))`; read kinds → `list_routes()`. (Prefer a catalog `write: bool` flag over a hardcoded kind list if the catalog supports it cleanly; otherwise match the known write kind.)

## 3. G5 — JWKS single-flight

- [ ] 3.1 Add a `tokio::sync::Mutex<()>` (or reuse the write lock as the refresh lock) in `JwksVerifier`. In `jwks()`, after a read-miss, acquire the refresh lock, **re-check** freshness under it (a prior holder may have refreshed), then fetch + cache once.
- [ ] 3.2 Preserve the empty-set poisoning guard and the 5s fetch timeout.

## 4. G6 — audit signature provenance

- [ ] 4.1 Add `signature_verified: bool` to `AuthContext` (from `OperatorContext`).
- [ ] 4.2 Include `signature_verified` in the runner's audit `tracing::info!` on dispatch/allow. Do not add the token/claims to any log field.

## 5. Verification

- [ ] 5.1 `cargo check/clippy/fmt` green (batched).
- [ ] 5.2 Unit/router tests: unauthenticated `/agui/stream` rejected; `application.deploy` → `Downstream` (no list_routes); JWKS concurrent cold-cache → single fetch (wiremock request count == 1); audit includes `signature_verified` (assert via a tracing capture or the AuthContext threading).
