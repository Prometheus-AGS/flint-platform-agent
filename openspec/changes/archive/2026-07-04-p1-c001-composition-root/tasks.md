## 1. Configuration layer

- [x] 1.1 Add a `config` module to `fpa-gateway`: typed `GatewayConfig` (forge URL, fabric endpoint, gate admin URL, bind addr) loaded from env with defaults.
- [x] 1.2 Validate required config at startup; fail fast with `anyhow` context (no silent defaults for security-relevant URLs).
- [x] 1.3 Add `reqwest = { version = "0.12", features = ["json","rustls-tls"] }` and `jsonwebtoken = "9"` to `fpa-gateway` (match flint-gate's stack).

## 2. Adapter construction + AppState

- [x] 2.1 Construct `ForgeAdapter`, `FabricAdapter`, `GateAdapter`, `McpClientAdapter` from `GatewayConfig`.
- [x] 2.2 Build `fpa_app::TaskRunner` from the four adapters (as `Arc<dyn Port>`).
- [x] 2.3 Define `AppState { runner: Arc<TaskRunner>, config: Arc<GatewayConfig> }`; inject via `Router::with_state`.
- [x] 2.4 Update the route modules (`agui`, `a2a`, `mcp`) to accept `State<Arc<AppState>>` (handlers still return stub payloads).

## 3. Gate identity extraction (consume only)

- [x] 3.1 Add an Axum extractor that reads gate-injected identity (gate-minted JWT / `X-User-*` headers) into an `OperatorContext` — **no Ory calls, no Ory JWKS**.
- [x] 3.2 Decode/validate the gate JWT with `jsonwebtoken` against gate's published key; map claims → roles/permissions. Treat absence as unauthenticated, not as authority.
- [x] 3.3 `tracing` span at the extractor boundary; never log the raw JWT or claims (CLAUDE.md gate).

## 4. Verification

- [x] 4.1 `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check` green.
- [x] 4.2 Unit test: `GatewayConfig` parses from env and rejects missing required vars.
- [x] 4.3 Unit test: gate-identity extractor maps a sample gate JWT to the expected `OperatorContext`; rejects a missing/invalid token.
- [x] 4.4 Smoke test: gateway boots with `AppState`, `/healthz` + the three surfaces still respond (run as in README).
