## Why

Four deferred debts from phase-4's security review and reflection are small,
self-contained, and security-relevant — grouped here so they close together before
the integration proof exercises them:

- **G4 / H2** — `GET /agui/stream` (`fpa-gateway::routes::agui::stream`) takes **no
  `OperatorContext`**: the SSE surface is unauthenticated. Anyone can open it.
- **G3** — the gate write-kind guard is misplaced: `task_runner.rs` routes **every**
  `TargetPort::Gate` kind (incl. `application.deploy`) to `list_routes()`, so a
  deploy silently *lists routes* instead of refusing.
- **G5 / H4** — `jwks::JwksVerifier::jwks()` drops the read guard, fetches, then
  takes the write guard: two concurrent callers on a cold/stale cache both hit the
  IdP (no single-flight). (Empty-set poisoning is already guarded.)
- **G6 / M2** — `identity.rs` already carries `signature_verified: bool`, but the
  **task audit** in `task_runner::run` never records it. The audit can't tell a
  gate-trusted identity from a signature-verified one.

## What Changes

- **G4:** add `_operator: OperatorContext` to the `agui::stream` handler so the
  extractor enforces identity (mirrors the phase-4 A2A `status`/`cancel` H1 fix).
  The stub still emits `run_start`/`run_end`; auth just gates entry.
- **G3:** in `task_runner`, branch on the task kind (or a catalog write-flag) for
  `TargetPort::Gate`: write kinds (`application.deploy`) return
  `PortError::Downstream("gate route-write not implemented")`; only read kinds call
  `list_routes()`.
- **G5:** serialize JWKS refresh with a `tokio::sync::Mutex` (single-flight):
  under the refresh lock, re-check freshness before fetching so a queued caller
  reuses the just-fetched set.
- **G6:** thread `signature_verified` from `OperatorContext` → `AuthContext` and
  include it in the runner's audit `tracing::info!` (never log the token/claims).

## Capabilities

### New Capabilities
- `security-debt-closure`: AG-UI stream authentication, a correct gate write-kind guard, single-flight JWKS refresh, and signature-verified provenance in the task audit.

### Modified Capabilities

## Impact

- `fpa-gateway` (`routes/agui.rs`, `jwks.rs`, `identity.rs` → `AuthContext`),
  `fpa-app` (`task_runner.rs` gate guard + audit field; `AuthContext` gains the flag).
- No new dependencies (`tokio::sync::Mutex` already present).

## Open Questions
- None. All four are localized fixes with source-verified sites (see assessment.md §2).
