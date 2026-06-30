## ADDED Requirements

### Requirement: MCP server over HTTP-Streaming only

The agent SHALL expose an MCP server using the streamable-HTTP transport at `POST /mcp` and MUST NOT offer a stdio transport for the server role.

#### Scenario: Streamable-HTTP available

- **WHEN** an MCP host connects to `/mcp` over streamable-HTTP and calls `initialize`
- **THEN** the server responds with its capabilities and server info

#### Scenario: No stdio server

- **WHEN** the agent's transports are inspected
- **THEN** the MCP server advertises only HTTP-Streaming and exposes no stdio server endpoint

### Requirement: Catalog tools exposed via MCP

The MCP server SHALL expose each A2A task-catalog `kind` as an MCP tool, and `tools/call` MUST dispatch through `TaskRunner` with the same permission enforcement as the A2A surface.

#### Scenario: Tools listed

- **WHEN** an MCP host calls `tools/list`
- **THEN** the response includes one tool per catalog task kind with its input schema

#### Scenario: Tool call routes through the runner

- **WHEN** an MCP host calls `tools/call` for a catalog tool
- **THEN** the call is dispatched through `TaskRunner`, enforcing gate-derived permissions
