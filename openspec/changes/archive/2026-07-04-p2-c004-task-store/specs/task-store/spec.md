## ADDED Requirements

### Requirement: Submitted tasks are recorded

The task store SHALL record each submitted task with its id and current state so
it can be queried afterward.

#### Scenario: Submit then query

- **WHEN** a task is submitted and later queried by its id
- **THEN** the store returns that task's recorded state

### Requirement: Status of unknown task

`GET /a2a/tasks/{id}` SHALL return a not-found response for an id the store does
not know.

#### Scenario: Unknown id

- **WHEN** a status query names an id with no stored task
- **THEN** the endpoint returns 404

### Requirement: Cancel transitions non-terminal tasks

Cancel SHALL move a non-terminal task to a canceled state and SHALL handle
terminal or unknown tasks explicitly (no silent success).

#### Scenario: Cancel a running task

- **WHEN** cancel is called on a non-terminal task
- **THEN** the task's stored state becomes canceled

#### Scenario: Cancel a completed task

- **WHEN** cancel is called on an already-terminal task
- **THEN** the response indicates the task cannot be canceled (not a false success)
