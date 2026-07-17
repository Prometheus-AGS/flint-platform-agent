## ADDED Requirements

### Requirement: mcp.tool.call invokes a downstream MCP tool for operators

`mcp.tool.call` SHALL be catalogued `TargetPort::Mcp` with `required_role: "operator"` and
a **non-empty** input schema requiring a `name` string and an `arguments` object. It SHALL
invoke the named tool on a downstream MCP server via `McpClient::call_tool(name, arguments)`
(real JSON-RPC `tools/call`). It is an **invoke**, not a read: `required_role` MUST be at
least `operator` (not `viewer`), and MUST NOT be `admin` (that tier is reserved for
gate/infra topology mutation such as `application.deploy`).

#### Scenario: Operator invokes an MCP tool

- **WHEN** `mcp.tool.call` runs with an operator role and input `{name, arguments}`
- **THEN** `McpClient::call_tool` is called with that exact `name` and `arguments` unaltered
- **AND** its return value is surfaced in the task result

#### Scenario: Sub-operator is denied before any invocation

- **WHEN** `mcp.tool.call` runs with a role below operator (e.g. viewer)
- **THEN** the permission check rejects it with `AppError::Unauthorized`
- **AND** `call_tool` is NOT called (the role gate precedes any port call — Base Rule 33)

#### Scenario: Malformed input is rejected by the schema

- **WHEN** `mcp.tool.call` runs with input missing `name` or `arguments`
- **THEN** the input-schema validation rejects it before any port call

### Requirement: TargetPort::Mcp dispatches by kind with a clean unknown-kind refusal

The runner's `TargetPort::Mcp` handling SHALL route by catalog kind through a `dispatch_mcp`
helper: `mcp.tool.list` → `list_tools`, `mcp.tool.call` → `call_tool`. An unknown Mcp kind
SHALL return a clean `PortError::Downstream` naming the kind and MUST NOT silently fall back
to `list_tools`. Every catalogued `TargetPort::Mcp` kind SHALL have a dispatch arm, enforced
by `every_mcp_catalog_kind_is_dispatched` (`MCP_KINDS = ["mcp.tool.list", "mcp.tool.call"]`).

#### Scenario: Unknown Mcp kind refuses cleanly

- **WHEN** an unrecognized Mcp kind is routed through `dispatch_mcp`
- **THEN** it returns a `Downstream` error naming the kind
- **AND** it does NOT fall back to `list_tools`

#### Scenario: Mcp kinds are dispatch-guarded

- **WHEN** the task-runner test suite runs
- **THEN** `every_mcp_catalog_kind_is_dispatched` asserts both `mcp.tool.list` and
  `mcp.tool.call` have dispatch arms, so a future undispatched Mcp kind fails the build

### Requirement: mcp.tool.call is audited without logging arguments and is non-idempotent

`mcp.tool.call` SHALL inherit the runner's audit pipeline: the allow and complete `tracing`
records SHALL carry `operator`, `kind`, the decision/outcome, and `signature_verified`
provenance, and SHALL NOT log the tool-call `arguments` payload, tokens, or claims (Base
Rule 34). `mcp.tool.call` SHALL be documented as **non-idempotent by default** (Base Rule
35): idempotency is a property of the downstream tool, not guaranteed by the agent, and the
audit record is the safety net. No dedup or idempotency-key machinery is introduced.

#### Scenario: Arguments are not emitted in logs

- **WHEN** `mcp.tool.call` is invoked and its audit records are captured
- **THEN** the emitted `tracing` records contain the operator, kind, and decision
- **AND** they do NOT contain the tool-call `arguments` payload
