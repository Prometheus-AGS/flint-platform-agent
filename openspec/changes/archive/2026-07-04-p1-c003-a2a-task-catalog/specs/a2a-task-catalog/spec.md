## ADDED Requirements

### Requirement: Administrative task catalog

The application layer SHALL maintain a typed catalog mapping each A2A task `kind` to an input schema and the port(s) it dispatches to. `TaskRunner::run` MUST validate the `kind` and input against the catalog before dispatch.

#### Scenario: Unknown kind

- **WHEN** a task with a `kind` not in the catalog is submitted
- **THEN** the runner returns `AppError::UnknownTaskKind` and dispatches to no port

#### Scenario: Invalid input

- **WHEN** a task's input fails its catalog input-schema validation
- **THEN** the runner returns `AppError::InvalidInput` and dispatches to no port

#### Scenario: Valid dispatch

- **WHEN** a known task with valid input is submitted
- **THEN** the runner dispatches to the mapped port and projects the result into a `TaskEvent`

### Requirement: Permission enforcement before dispatch

The runner SHALL enforce gate-derived permissions before executing any task and MUST NOT call a port when permission is denied.

#### Scenario: Denied task

- **WHEN** the operator lacks permission for the task's `kind`
- **THEN** the runner returns `TaskEvent::Failed` with a reason and calls no port

### Requirement: Per-task audit record

Each task execution SHALL emit an audit record capturing the request, kind, decision, and outcome, and MUST NOT include secrets or raw claims.

#### Scenario: Audited execution

- **WHEN** any task runs to completion or failure
- **THEN** an audit record is emitted that excludes JWT payloads and tenant identifiers
