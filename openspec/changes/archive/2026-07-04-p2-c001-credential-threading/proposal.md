## Why

To read fabric data, `fpa-forge` must call forge's `/graphql` with the operator's
**gate-issued JWT as the `Authorization` bearer** — forge runs `rls_from_bearer`
and applies RLS. Today the c001 gate extractor **verifies the token but discards
the raw string**, and the ports carry no per-request credential, so there is no
way to forward it. This change threads the raw bearer from the surfaces through
the app layer to the ports, without fabricating or logging it.

## What Changes

- Retain the raw bearer in the gateway's `OperatorContext` (from the gate JWT / gate headers).
- Add `bearer: Option<String>` to `fpa_app::AuthContext` (decision: `AuthContext.bearer` over a separate request-context — smallest change for one downstream that needs it).
- Pass the credential from `TaskRunner::run` to the port(s) that need it (the forge port method gains access to the bearer).
- Propagate gate identity into the **MCP** surface: `tools/call` builds `AuthContext` (incl. bearer) from the caller's gate identity on `POST /mcp`, replacing the hardcoded `viewer+operator`.
- Never log the bearer (extend the c003 `skip_all` audit discipline to the forge adapter and the MCP path).

## Capabilities

### New Capabilities
- `credential-threading`: Per-request forwarding of the operator's gate bearer from the AG-UI/A2A/MCP surfaces through `TaskRunner` to downstream ports, with strict no-log handling.

### Modified Capabilities

## Impact

- `fpa-app` (`AuthContext.bearer`, `TaskRunner::run` signature), `fpa-ports` (forge port method may take the bearer), `fpa-gateway` (retain raw bearer in `OperatorContext`; MCP `tools/call` builds real `AuthContext`).
- No new dependencies.
- Blocks `p2-c002` (forge integration needs the bearer).

## Open Questions
- **Q4 (MCP caller identity):** does `POST /mcp` carry a gate JWT (Authorization header)? If yes, reuse the c001 extractor for MCP. If MCP hosts cannot send one, the MCP surface keeps a scoped-down default and this debt stays partially open. **Confirm at execute.**
