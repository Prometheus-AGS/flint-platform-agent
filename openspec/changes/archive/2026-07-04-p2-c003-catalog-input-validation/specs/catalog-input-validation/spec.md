## ADDED Requirements

### Requirement: Per-kind input schema

Each catalog entry SHALL declare an input JSON Schema describing its accepted
arguments.

#### Scenario: Schema surfaced via MCP

- **WHEN** an MCP host calls `tools/list`
- **THEN** each tool's `inputSchema` reflects its catalog entry's declared schema, not a placeholder

### Requirement: Inputs validated before dispatch

`TaskRunner::run` SHALL validate `task.input` against the catalogued schema before
any port call, returning `AppError::InvalidInput` on mismatch and dispatching to
no port.

#### Scenario: Invalid input rejected

- **WHEN** a task's input omits a required field for its kind
- **THEN** the runner returns `AppError::InvalidInput` and calls no port

#### Scenario: Valid input proceeds

- **WHEN** a task's input satisfies its kind's schema
- **THEN** validation passes and dispatch proceeds
