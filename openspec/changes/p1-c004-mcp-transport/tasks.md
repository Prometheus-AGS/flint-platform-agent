## 1. rmcp version decision (Open Question — resolve first)

- [x] 1.1 Confirm the canonical `rmcp` version line (crates.io 2.0.0 vs docs.rs 1.7.0) and the streamable-http server feature flag; record in the change + KBD decision-log.
- [x] 1.2 Add `rmcp` with the resolved version + features (server, client, transport-streamable-http) to `fpa-mcp` and `fpa-gateway`.

## 2. MCP server (HTTP-Streaming only)

- [x] 2.1 Replace `fpa-gateway/routes/mcp.rs` hand-rolled JSON-RPC with `rmcp`'s streamable-HTTP server mounted at `/mcp`. **No stdio transport.**
- [x] 2.2 Expose the A2A task catalog (`p1-c003`) as MCP tools: each `TaskKind` → one tool with its input schema.
- [x] 2.3 `tools/call` dispatches through `TaskRunner` (same path as A2A), enforcing gate-derived permissions; stream large results over the HTTP-Streaming transport.

## 3. MCP client

- [x] 3.1 Implement `fpa-mcp::McpClientAdapter` over `rmcp`'s client: `list_tools` + `call_tool` against a downstream MCP server endpoint.
- [x] 3.2 Allow composing multiple downstream servers (config-driven endpoint list).

## 4. Skill format for tools

- [x] 4.1 Define the `SKILL.md`-style skill convention for MCP tools (mirror Prometheus/Anthropic skill format); document where skills live and how they register as tools.
- [x] 4.2 Provide one example skill that maps to a catalog task end-to-end.

## 5. Verification

- [x] 5.1 `cargo check/clippy/fmt` green.
- [x] 5.2 Server: `initialize` + `tools/list` return the catalog tools; `tools/call` for a known tool routes through `TaskRunner` (fake adapters).
- [x] 5.3 Client: `list_tools`/`call_tool` against a local fake MCP server succeed.
- [x] 5.4 Confirm the server advertises only HTTP-Streaming (no stdio) and rejects unknown methods with proper JSON-RPC errors.
