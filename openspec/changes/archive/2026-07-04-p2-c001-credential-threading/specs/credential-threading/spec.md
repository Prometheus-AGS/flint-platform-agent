## ADDED Requirements

### Requirement: Operator bearer is carried, not fabricated

The gateway SHALL retain the operator's raw gate-issued bearer alongside the
derived `OperatorContext`, and the application layer SHALL carry it as an optional
credential on `AuthContext`. The agent MUST NOT synthesize or alter the bearer.

#### Scenario: Bearer available downstream

- **WHEN** a request arrives with a valid gate bearer
- **THEN** the resulting `AuthContext` carries that exact bearer for downstream forwarding

#### Scenario: No bearer present

- **WHEN** a request arrives without a gate bearer
- **THEN** `AuthContext` carries no bearer and downstream calls requiring RLS are not made with a fabricated credential

### Requirement: Bearer reaches the port that needs it

`TaskRunner` SHALL make the operator's bearer available to the port method it
dispatches to, so an RLS-enforcing downstream (forge) receives the operator's
identity.

#### Scenario: Forge-targeted task forwards the bearer

- **WHEN** a forge-targeted task runs with a bearer in `AuthContext`
- **THEN** the forge port invocation is able to send that bearer as the `Authorization` header

### Requirement: Bearer is never logged

No log line, span field, or audit record SHALL contain the bearer token or its
claims.

#### Scenario: Audit excludes the token

- **WHEN** any task runs (allowed or denied)
- **THEN** emitted logs and audit records contain no bearer string and no raw claims

### Requirement: MCP calls carry real caller identity

The MCP `tools/call` path SHALL build `AuthContext` from the caller's gate
identity rather than a hardcoded role set, when the MCP transport presents one.

#### Scenario: MCP caller with gate identity

- **WHEN** an MCP `tools/call` arrives with a gate identity on `POST /mcp`
- **THEN** the task runs under that caller's roles and bearer, not a constant default
