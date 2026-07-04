## ADDED Requirements

### Requirement: An automated smoke drives the live agent end-to-end

A Playwright HTTP smoke SHALL run against the containerized agent at
`http://localhost:8088` and assert the full operator flow: health, authentication
(including the `/agui/stream` auth gate), project create → inspect → list via the
store, `fabric.health`, and MCP `tools/list` + `tools/call`. It MUST fail if any hop
returns an unexpected status/body.

#### Scenario: Happy-path smoke passes

- **WHEN** the agent stack is up and the smoke runs with a valid minted HS256 bearer
- **THEN** healthz is 200, `project.create`→`inspect`→`list` round-trips the project through the store, `fabric.health` is ok, MCP `tools/list`+`tools/call` succeed, and authenticated `/agui/stream` opens

#### Scenario: Unauthenticated is rejected

- **WHEN** a protected endpoint (e.g. `/agui/stream`, `POST /a2a/tasks`) is called with no bearer
- **THEN** the agent returns 401 and the smoke asserts it

### Requirement: The smoke is one command, self-contained, and cleans up

`smoke/run.sh` SHALL bring the compose stack up, wait for `:8088/healthz`, run the
Playwright smoke, and tear the stack down (`down -v`) on both success and failure.

#### Scenario: One-command run

- **WHEN** `smoke/run.sh` is executed on a machine with Docker healthy
- **THEN** it builds+starts the stack, runs the smoke to a pass/fail result, and removes the containers afterward
