## ADDED Requirements

### Requirement: Downstream MCP client

The `fpa-mcp` adapter SHALL implement an MCP client capable of listing and calling tools on downstream MCP servers, composing their tools into the agent's toolset.

#### Scenario: List downstream tools

- **WHEN** the client connects to a configured downstream MCP server
- **THEN** `list_tools` returns that server's advertised tools

#### Scenario: Call downstream tool

- **WHEN** the client invokes `call_tool` with a valid tool name and arguments
- **THEN** the downstream server's result is returned to the caller

### Requirement: Multiple downstream servers

The client SHALL support composing tools from more than one configured downstream MCP server.

#### Scenario: Aggregated toolset

- **WHEN** two downstream servers are configured
- **THEN** the agent can list and call tools from either server
