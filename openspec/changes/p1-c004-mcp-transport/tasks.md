## 1. Transport decision (Open Question — RESOLVED: use canonical JSON-RPC, NOT rmcp)

- [x] 1.1 Decision: the canonical Prometheus `mcp-server` skill uses hand-rolled JSON-RPC 2.0 over Axum, not rmcp. Reversed the analyze/spec "adopt rmcp" verdict (recorded in decision-log).
- [x] 1.2 No rmcp dependency added; server + client both use the JSON-RPC-over-HTTP house pattern (server on Axum, client on reqwest).

## 2. MCP server (HTTP-Streaming only)

- [x] 2.1 Extend `fpa-gateway/routes/mcp.rs` hand-rolled JSON-RPC (per mcp-server skill); mounted at `/mcp`. **No stdio transport.**
- [x] 2.2 Expose the A2A task catalog as MCP tools via `tool_definitions()` (one tool per kind; minimal object schema — per-kind input schema is a carry-forward).
- [x] 2.3 `tools/call` dispatches through `TaskRunner` (same permission + audit path as A2A). (Response streaming for large results: not needed at current payload sizes — carry-forward.)

## 3. MCP client

- [x] 3.1 Implement `fpa-mcp::McpClientAdapter` as JSON-RPC-over-reqwest: `list_tools` + `call_tool`.
- [ ] 3.2 Multiple downstream servers (config-driven endpoint list) — NOT done; single endpoint per adapter this phase (carry-forward).

## 4. Skill format for tools

- [x] 4.1 Define the `SKILL.md`-style skill convention for MCP tools (mirror Prometheus/Anthropic skill format); document where skills live and how they register as tools.
- [x] 4.2 Provide one example skill that maps to a catalog task end-to-end.

## 5. Verification

- [x] 5.1 `cargo check/clippy/fmt` green.
- [x] 5.2 Server: `initialize` + `tools/list` return the catalog tools; `tools/call` for a known tool routes through `TaskRunner` (fake adapters).
- [~] 5.3 Client: only the no-endpoint transport-error path is unit-tested; a round-trip against a live fake MCP server is NOT yet covered (carry-forward).
- [x] 5.4 Confirm the server advertises only HTTP-Streaming (no stdio) and rejects unknown methods/tools with proper JSON-RPC errors (verified live: unknown method → -32601, unknown tool → -32601, permission denial → isError).
