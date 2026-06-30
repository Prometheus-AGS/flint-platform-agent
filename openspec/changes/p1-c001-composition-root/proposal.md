## Why

The gateway mounts protocol routes (AG-UI, A2A, MCP) but they are stateless stubs — no adapter is wired and `fpa_app::TaskRunner` is never constructed. Nothing connects the protocol surfaces to the fabric. This change builds the composition root so subsequent changes (task catalog, MCP transport) have a place to plug in.

## What Changes

- Construct the four plane adapters (`fpa-forge`, `fpa-fabric`, `fpa-gate`, `fpa-mcp`) in `fpa-gateway` from configuration (endpoints, gate admin URL).
- Build `fpa_app::TaskRunner` from those adapters and inject it as Axum shared state (`State<Arc<AppState>>`).
- Add a typed configuration layer (env-driven; e.g. `FPA_FORGE_URL`, `FPA_GATE_ADMIN_URL`, `FPA_FABRIC_ENDPOINT`) with startup validation.
- Thread the gate-injected operator identity into request handling as the auth context (consume gate JWT/headers; **no Ory calls**).
- Keep adapters returning `todo!()`/`PortError` — this change wires, it does not implement downstream calls.

## Capabilities

### New Capabilities
- `gateway-composition`: Configuration loading, adapter construction, `TaskRunner` assembly, Axum state injection, and gate-identity extraction as the agent's composition root.

### Modified Capabilities

## Impact

- `fpa-gateway` (composition root — the only crate importing concrete adapters), new `config` + `state` modules.
- Adds deps to `fpa-gateway`: `reqwest` (gate/forge HTTP), `jsonwebtoken` (read gate JWT). No Ory crates.
- No change to the hexagonal dependency rule: domain/app still import no adapter.
