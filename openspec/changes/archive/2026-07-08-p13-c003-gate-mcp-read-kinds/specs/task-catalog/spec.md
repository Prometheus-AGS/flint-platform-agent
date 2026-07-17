## ADDED Requirements

### Requirement: gate.route.list enumerates gate routes for operators

`gate.route.list` SHALL be catalogued `TargetPort::Gate` with `required_role: "operator"`
and an empty input schema. It SHALL return the routes from `GateAdmin::list_routes`,
reached via flint-gate's admin-port adapter. It MUST NOT be treated as a write and MUST NOT
call Ory directly — flint-gate remains the only auth boundary.

#### Scenario: Operator lists gate routes

- **WHEN** `gate.route.list` runs with an operator role
- **THEN** `list_routes` is called and its routes are returned
- **AND** the kind is classified as a Gate read, not a write refusal

#### Scenario: Non-operator is denied

- **WHEN** `gate.route.list` runs with a role below operator (e.g. viewer)
- **THEN** the permission check rejects it with `AppError::Unauthorized`
- **AND** `list_routes` is not called

### Requirement: mcp.tool.list lists downstream MCP tools for viewers

`mcp.tool.list` SHALL be catalogued `TargetPort::Mcp` with `required_role: "viewer"` and an
empty input schema. It SHALL return the tools from `McpClient::list_tools`. `call_tool`
(an invoke/write) MUST remain out of the catalog this phase.

#### Scenario: Viewer lists MCP tools

- **WHEN** `mcp.tool.list` runs with a viewer role
- **THEN** `list_tools` is called and the tool list is returned

#### Scenario: MCP kinds are dispatch-guarded

- **WHEN** the task-runner test suite runs
- **THEN** `every_mcp_catalog_kind_is_dispatched` asserts every catalogued `TargetPort::Mcp`
  kind has a dispatch arm, so a future undispatched Mcp kind fails the build

### Requirement: GATE_READ_KINDS classifies gate reads

The runner's Gate classification SHALL list every catalogued Gate **read** kind in
`GATE_READ_KINDS`, and `every_gate_catalog_kind_is_classified` SHALL fail if a Gate catalog
kind is neither a known write nor a listed read.

#### Scenario: gate.route.list is a classified read

- **WHEN** the Gate classification guard runs
- **THEN** `gate.route.list` is present in `GATE_READ_KINDS`
- **AND** `application.deploy` remains classified as a write
