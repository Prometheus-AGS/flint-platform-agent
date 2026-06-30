## Why

The MCP server is a hand-rolled JSON-RPC stub and the MCP client (`fpa-mcp`) is a trait with no transport. Requirements #3 (skills/tools over MCP) and #4 (MCP client) need a real transport. Analyze selected the official SDK (`rmcp`) which covers both roles and our HTTP-Streaming-only server constraint from one dependency.

## What Changes

- Adopt `rmcp` (official MCP Rust SDK) for both the MCP **server** (HTTP-Streaming transport only — no stdio, per the project constraint) and the MCP **client** (`fpa-mcp` adapter).
- Replace the hand-rolled JSON-RPC handler in `fpa-gateway/routes/mcp.rs` with `rmcp`'s streamable-HTTP server, exposing the A2A task catalog (`p1-c003`) as MCP tools.
- Implement `fpa-mcp::McpClientAdapter` over `rmcp`'s client to compose downstream MCP servers' tools.
- Define the **skill format** for MCP tools (mirror the Prometheus/Anthropic `SKILL.md` convention) and a registration path from skills → exposed tools.

## Capabilities

### New Capabilities
- `mcp-server-http`: The agent's MCP server over streamable-HTTP, exposing fabric administrative tools (backed by the A2A catalog).
- `mcp-client`: Downstream MCP client transport composing external servers' tools.

### Modified Capabilities

## Impact

- `fpa-mcp` (client transport), `fpa-gateway` (server replaces hand-rolled JSON-RPC).
- New dep (pending Open Question): `rmcp` with the streamable-http + server + client features.
- Depends on `p1-c003` (tools surface the task catalog).

## Open Questions

- **`rmcp` version line:** crates.io shows `2.0.0`, docs.rs shows `1.7.0`. Confirm the canonical/maintained line and the exact streamable-http server feature flag before pinning (Base Rule 22). **Resolve before adding the dependency.**
- Stdio is explicitly **out of scope** for the server (HTTP-Streaming only); the client may use whatever transport a downstream server requires.
